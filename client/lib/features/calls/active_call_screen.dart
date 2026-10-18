import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';
import '../../core/network/api_client.dart';
import '../../core/services/call_signaling_service.dart';
import '../../core/services/sound_service.dart';
import '../../core/services/webrtc_service.dart';
import '../../core/theme/app_theme.dart';
import '../auth/auth_state.dart';

enum CallStatus { connecting, ringing, connected, ended }

class ActiveCallScreen extends ConsumerStatefulWidget {
  final String? callId;
  final String recipientDid;
  final String recipientName;
  final bool isVideoCall;
  final bool isIncoming;

  const ActiveCallScreen({
    super.key,
    this.callId,
    required this.recipientDid,
    required this.recipientName,
    this.isVideoCall = false,
    this.isIncoming = false,
  });

  @override
  ConsumerState<ActiveCallScreen> createState() => _ActiveCallScreenState();
}

class _ActiveCallScreenState extends ConsumerState<ActiveCallScreen> {
  CallStatus _status = CallStatus.connecting;
  bool _isMuted = false;
  bool _isVideoOff = false;
  bool _isSpeakerOn = true;
  bool _isFrontCamera = true;
  int _durationSeconds = 0;
  String _statusMessage = '';
  late String _callId;
  Timer? _callDurationTimer;
  Timer? _signalingPollTimer;

  final WebRTCService _webrtc = WebRTCService();
  final Set<String> _processedIceCandidateIds = {};
  bool _hasProcessedRemoteAnswer = false;
  bool _hasRemoteVideo = false;

  @override
  void initState() {
    super.initState();
    _isVideoOff = !widget.isVideoCall;

    // Inform call signaling manager that active call screen is mounted
    CallSignalingService().setIsInCall(true);

    final currentUser = ref.read(authProvider).user;
    final myDid = currentUser?.did ?? 'did:bharat:unknown';
    final myName = currentUser?.name ?? myDid;

    _callId = widget.callId ?? 'call_${DateTime.now().millisecondsSinceEpoch}_$myDid';

    _setupWebRtcCallbacks(myDid);

    if (widget.isIncoming) {
      _status = CallStatus.connected;
      _startDurationTimer();
      _initIncomingCallFlow(myDid, myName);
      _startSignalingPolling(myDid, forIncoming: true);
    } else {
      _status = CallStatus.ringing;
      SoundService().startRinging();
      _initOutgoingCallFlow(myDid, myName);
      _startSignalingPolling(myDid, forIncoming: false);
    }
  }

  void _setupWebRtcCallbacks(String myDid) {
    _webrtc.onLocalCandidate = (RTCIceCandidate candidate) {
      if (candidate.candidate == null || candidate.candidate!.isEmpty) return;
      ref.read(apiClientProvider).sendCallSignal(
        callId: _callId,
        callerDid: myDid,
        recipientDid: widget.recipientDid,
        signalType: 'ICE_CANDIDATE',
        callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
        candidate: candidate.candidate,
        sdpMid: candidate.sdpMid,
        sdpMLineIndex: candidate.sdpMLineIndex,
      );
    };

    _webrtc.onRemoteStream = (MediaStream stream) {
      if (mounted) {
        setState(() {
          _hasRemoteVideo = stream.getVideoTracks().isNotEmpty;
        });
      }
    };
  }

  Future<void> _initIncomingCallFlow(String myDid, String myName) async {
    await _webrtc.initialize(isVideoCall: widget.isVideoCall);

    // Retrieve caller's OFFER SDP
    final signals = await ref.read(apiClientProvider).getCallSignals(
      did: myDid,
      callId: _callId,
    );

    final offerSignal = signals.firstWhere(
      (s) => s['signal_type'] == 'OFFER' || (s['sdp'] != null && s['status'] == 'RINGING'),
      orElse: () => signals.isNotEmpty ? signals.first : <String, dynamic>{},
    );

    final offerSdp = offerSignal['sdp'] as String?;
    String? answerSdp;

    if (offerSdp != null && offerSdp.isNotEmpty) {
      answerSdp = await _webrtc.createAnswer(offerSdp);
    }

    // Send ANSWER signal with SDP back to caller
    await ref.read(apiClientProvider).sendCallSignal(
      callId: _callId,
      callerDid: widget.recipientDid,
      recipientDid: myDid,
      signalType: 'ANSWER',
      callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
      sdp: answerSdp,
    );

    if (mounted) setState(() {});
  }

  Future<void> _initOutgoingCallFlow(String myDid, String myName) async {
    // 1. Immediately send initial OFFER signal so other device starts ringing without delay
    await ref.read(apiClientProvider).sendCallSignal(
      callId: _callId,
      callerDid: myDid,
      callerName: myName,
      recipientDid: widget.recipientDid,
      signalType: 'OFFER',
      callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
    );

    // 2. Initialize WebRTC hardware media in background and send SDP update
    try {
      await _webrtc.initialize(isVideoCall: widget.isVideoCall);
      final offerSdp = await _webrtc.createOffer(isVideoCall: widget.isVideoCall);
      if (offerSdp != null && offerSdp.isNotEmpty) {
        await ref.read(apiClientProvider).sendCallSignal(
          callId: _callId,
          callerDid: myDid,
          recipientDid: widget.recipientDid,
          signalType: 'OFFER',
          callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
          sdp: offerSdp,
        );
      }
    } catch (_) {}

    if (mounted) setState(() {});
  }

  void _startDurationTimer() {
    _callDurationTimer?.cancel();
    _callDurationTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) {
        setState(() => _durationSeconds++);
      }
    });
  }

  void _startSignalingPolling(String myDid, {required bool forIncoming}) {
    _signalingPollTimer?.cancel();
    _signalingPollTimer = Timer.periodic(const Duration(milliseconds: 700), (_) async {
      if (!mounted || _status == CallStatus.ended) return;

      try {
        final signals = await ref.read(apiClientProvider).getCallSignals(
          did: myDid,
          callId: _callId,
        );

        if (signals.isEmpty) return;

        // Check overall call status from main signal
        final mainSignal = signals.firstWhere(
          (s) => s['signal_type'] != 'ICE_CANDIDATE',
          orElse: () => signals.first,
        );
        final status = (mainSignal['status'] as String? ?? '').toUpperCase();

        if (forIncoming) {
          if (status == 'ENDED' || status == 'REJECTED') {
            _handleRemoteHangup();
            return;
          }
        } else {
          if (_status == CallStatus.ringing || _status == CallStatus.connecting) {
            if (status == 'CONNECTED') {
              if (mounted) {
                setState(() {
                  _status = CallStatus.connected;
                  _statusMessage = '';
                });
                SoundService().stopRinging();
                _startDurationTimer();
              }

              // Process remote Answer SDP
              if (!_hasProcessedRemoteAnswer) {
                final answerSdp = mainSignal['sdp'] as String?;
                if (answerSdp != null && answerSdp.isNotEmpty) {
                  _hasProcessedRemoteAnswer = true;
                  await _webrtc.handleRemoteAnswer(answerSdp);
                }
              }
            } else if (status == 'REJECTED') {
              _handleCallDeclined();
              return;
            } else if (status == 'ENDED') {
              _handleRemoteHangup();
              return;
            }
          } else if (_status == CallStatus.connected) {
            if (status == 'ENDED') {
              _handleRemoteHangup();
              return;
            }
          }
        }

        // Process incoming remote ICE candidates
        for (final s in signals) {
          if (s['signal_type'] == 'ICE_CANDIDATE' &&
              s['candidate'] != null &&
              s['caller_did'] == widget.recipientDid) {
            final candStr = s['candidate'] as String;
            final mid = s['sdp_mid'] as String?;
            final mLine = s['sdp_mline_index'] as int?;
            final key = '$candStr-$mid-$mLine';
            if (!_processedIceCandidateIds.contains(key)) {
              _processedIceCandidateIds.add(key);
              await _webrtc.addIceCandidate(
                candidate: candStr,
                sdpMid: mid,
                sdpMLineIndex: mLine,
              );
            }
          }
        }
      } catch (_) {}
    });
  }

  void _handleCallDeclined() {
    if (_status == CallStatus.ended) return;
    _signalingPollTimer?.cancel();
    _callDurationTimer?.cancel();
    SoundService().stopRinging();
    SoundService().playCallEnded();

    if (mounted) {
      setState(() {
        _status = CallStatus.ended;
        _statusMessage = 'Call Declined';
      });

      Future.delayed(const Duration(milliseconds: 1400), () {
        if (mounted) {
          Navigator.of(context).pop();
        }
      });
    }
  }

  void _handleRemoteHangup() {
    if (_status == CallStatus.ended) return;
    _signalingPollTimer?.cancel();
    _callDurationTimer?.cancel();
    SoundService().stopRinging();
    SoundService().playCallEnded();

    if (mounted) {
      setState(() {
        _status = CallStatus.ended;
        _statusMessage = 'Call Ended';
      });

      Future.delayed(const Duration(milliseconds: 1000), () {
        if (mounted) {
          Navigator.of(context).pop();
        }
      });
    }
  }

  Future<void> _endCall() async {
    if (_status == CallStatus.ended) return;
    final wasRinging = (_status == CallStatus.ringing || _status == CallStatus.connecting);
    setState(() {
      _status = CallStatus.ended;
      _statusMessage = wasRinging ? 'Cancelled' : 'Call Ended';
    });

    _signalingPollTimer?.cancel();
    _callDurationTimer?.cancel();
    SoundService().stopRinging();
    await SoundService().playCallEnded();

    final myDid = ref.read(authProvider).user?.did ?? 'did:bharat:unknown';

    // Send CANCEL if still ringing, or HANGUP if connected
    await ref.read(apiClientProvider).sendCallSignal(
      callId: _callId,
      callerDid: myDid,
      recipientDid: widget.recipientDid,
      signalType: wasRinging ? 'CANCEL' : 'HANGUP',
      callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
    );

    // Record persistent call log
    await ref.read(apiClientProvider).recordCallLog(
      callerDid: widget.isIncoming ? widget.recipientDid : myDid,
      recipientDid: widget.isIncoming ? myDid : widget.recipientDid,
      callType: widget.isVideoCall ? 'VIDEO' : 'AUDIO',
      status: wasRinging ? 'MISSED' : 'COMPLETED',
      durationSeconds: _durationSeconds > 0 ? _durationSeconds : (wasRinging ? 0 : 1),
    );

    await _webrtc.dispose();

    if (mounted) {
      Navigator.of(context).pop();
    }
  }

  String _formatDuration(int seconds) {
    final mins = (seconds ~/ 60).toString().padLeft(2, '0');
    final secs = (seconds % 60).toString().padLeft(2, '0');
    return '$mins:$secs';
  }

  @override
  void dispose() {
    CallSignalingService().setIsInCall(false);
    _signalingPollTimer?.cancel();
    _callDurationTimer?.cancel();
    SoundService().stopRinging();
    _webrtc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final displayName = widget.recipientName.isNotEmpty ? widget.recipientName : widget.recipientDid;

    return Scaffold(
      backgroundColor: const Color(0xFF0D141C),
      body: SafeArea(
        child: Stack(
          children: [
            // Remote Video Background (or connecting video placeholder)
            if (widget.isVideoCall && !_isVideoOff)
              Positioned.fill(
                child: (_webrtc.hasRemoteStream && _hasRemoteVideo)
                    ? RTCVideoView(
                        _webrtc.remoteRenderer,
                        objectFit: RTCVideoViewObjectFit.RTCVideoViewObjectFitCover,
                      )
                    : Container(
                        decoration: const BoxDecoration(
                          gradient: LinearGradient(
                            colors: [Color(0xFF1A2634), Color(0xFF0F1722)],
                            begin: Alignment.topCenter,
                            end: Alignment.bottomCenter,
                          ),
                        ),
                        child: Center(
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              CircleAvatar(
                                radius: 54,
                                backgroundColor: AppTheme.surfaceDark,
                                child: Text(
                                  displayName.isNotEmpty ? displayName[0].toUpperCase() : "B",
                                  style: const TextStyle(fontSize: 48, fontWeight: FontWeight.bold, color: Colors.white),
                                ),
                              ),
                              const SizedBox(height: 16),
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                                decoration: BoxDecoration(
                                  color: Colors.black.withOpacity(0.6),
                                  borderRadius: BorderRadius.circular(16),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    const SizedBox(
                                      width: 14,
                                      height: 14,
                                      child: CircularProgressIndicator(strokeWidth: 2, color: AppTheme.primaryGreen),
                                    ),
                                    const SizedBox(width: 8),
                                    Text(
                                      _status == CallStatus.connected
                                          ? "Connecting Remote Video Stream..."
                                          : "Ringing Remote Enclave...",
                                      style: const TextStyle(color: Colors.white, fontSize: 12),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
              ),

            // Top Header: Name, Status, SFrame cryptographic badge
            Positioned(
              top: 30,
              left: 20,
              right: 20,
              child: Column(
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                    decoration: BoxDecoration(
                      color: AppTheme.surfaceDark.withOpacity(0.8),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: const Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.lock, size: 12, color: AppTheme.primaryGreen),
                        SizedBox(width: 4),
                        Text(
                          "End-to-End Encrypted (WebRTC DTLS-SRTP)",
                          style: TextStyle(color: AppTheme.primaryGreen, fontSize: 11),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    displayName,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    _statusMessage.isNotEmpty
                        ? _statusMessage
                        : _status == CallStatus.connecting
                            ? "Connecting..."
                            : _status == CallStatus.ringing
                                ? "Ringing..."
                                : _status == CallStatus.connected
                                    ? _formatDuration(_durationSeconds)
                                    : "Call Ended",
                    style: TextStyle(
                      color: _status == CallStatus.connected ? AppTheme.primaryGreen : AppTheme.textSecondary,
                      fontSize: 15,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ],
              ),
            ),

            // Center Avatar (when audio call or video is turned off)
            if (!widget.isVideoCall || _isVideoOff)
              Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    CircleAvatar(
                      radius: 64,
                      backgroundColor: AppTheme.surfaceDark,
                      child: Text(
                        displayName.isNotEmpty ? displayName[0].toUpperCase() : "B",
                        style: const TextStyle(fontSize: 48, fontWeight: FontWeight.bold, color: Colors.white),
                      ),
                    ),
                    const SizedBox(height: 16),
                    Text(
                      widget.recipientDid,
                      style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12, fontFamily: 'monospace'),
                    ),
                    if (_status == CallStatus.connected) ...[
                      const SizedBox(height: 12),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                        decoration: BoxDecoration(
                          color: AppTheme.surfaceDark.withOpacity(0.8),
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.4)),
                        ),
                        child: const Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.mic, size: 14, color: AppTheme.primaryGreen),
                            SizedBox(width: 6),
                            Text(
                              "Voice Connected (WebRTC 48kHz Opus)",
                              style: TextStyle(color: AppTheme.primaryGreen, fontSize: 12),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ],
                ),
              ),

            // Local Video Preview (Picture-in-Picture Floating Window)
            if (widget.isVideoCall && !_isVideoOff)
              Positioned(
                top: 90,
                right: 16,
                width: 110,
                height: 160,
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(14),
                  child: Container(
                    decoration: BoxDecoration(
                      color: AppTheme.surfaceDark,
                      border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.6), width: 1.5),
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withOpacity(0.5),
                          blurRadius: 10,
                          offset: const Offset(0, 4),
                        ),
                      ],
                    ),
                    child: _webrtc.hasLocalStream
                        ? RTCVideoView(
                            _webrtc.localRenderer,
                            mirror: _isFrontCamera,
                            objectFit: RTCVideoViewObjectFit.RTCVideoViewObjectFitCover,
                          )
                        : const Center(
                            child: Icon(Icons.person, color: AppTheme.primaryGreen, size: 36),
                          ),
                  ),
                ),
              ),

            // Bottom Controls Bar
            Positioned(
              bottom: 40,
              left: 20,
              right: 20,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
                decoration: BoxDecoration(
                  color: AppTheme.surfaceDark.withOpacity(0.9),
                  borderRadius: BorderRadius.circular(36),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.4),
                      blurRadius: 16,
                      offset: const Offset(0, 4),
                    ),
                  ],
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                  children: [
                    // Mute Microphone
                    IconButton(
                      icon: Icon(
                        _isMuted ? Icons.mic_off : Icons.mic,
                        color: _isMuted ? Colors.redAccent : Colors.white,
                        size: 26,
                      ),
                      tooltip: _isMuted ? "Unmute Mic" : "Mute Mic",
                      onPressed: () {
                        setState(() {
                          _isMuted = !_isMuted;
                          _webrtc.setMicrophoneMute(_isMuted);
                        });
                      },
                    ),

                    // Toggle Video Camera
                    if (widget.isVideoCall) ...[
                      IconButton(
                        icon: Icon(
                          _isVideoOff ? Icons.videocam_off : Icons.videocam,
                          color: _isVideoOff ? Colors.redAccent : Colors.white,
                          size: 26,
                        ),
                        tooltip: _isVideoOff ? "Turn On Camera" : "Turn Off Camera",
                        onPressed: () {
                          setState(() {
                            _isVideoOff = !_isVideoOff;
                            _webrtc.setCameraEnabled(!_isVideoOff);
                          });
                        },
                      ),
                      IconButton(
                        icon: const Icon(Icons.flip_camera_ios, color: Colors.white, size: 24),
                        tooltip: "Switch Camera",
                        onPressed: () async {
                          final flipped = await _webrtc.switchCamera();
                          if (mounted) setState(() => _isFrontCamera = flipped);
                        },
                      ),
                    ],

                    // Speakerphone Toggle
                    IconButton(
                      icon: Icon(
                        _isSpeakerOn ? Icons.volume_up : Icons.volume_down,
                        color: _isSpeakerOn ? AppTheme.primaryGreen : Colors.white,
                        size: 26,
                      ),
                      tooltip: _isSpeakerOn ? "Speaker Active" : "Earpiece Active",
                      onPressed: () {
                        setState(() => _isSpeakerOn = !_isSpeakerOn);
                      },
                    ),

                    // End Call Button (Big Red Button)
                    GestureDetector(
                      onTap: _endCall,
                      child: Container(
                        padding: const EdgeInsets.all(12),
                        decoration: const BoxDecoration(
                          color: Colors.redAccent,
                          shape: BoxShape.circle,
                        ),
                        child: const Icon(Icons.call_end, color: Colors.white, size: 26),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

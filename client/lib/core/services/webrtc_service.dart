import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';

typedef IceCandidateCallback = void Function(RTCIceCandidate candidate);
typedef RemoteStreamCallback = void Function(MediaStream stream);

class WebRTCService {
  final RTCVideoRenderer localRenderer = RTCVideoRenderer();
  final RTCVideoRenderer remoteRenderer = RTCVideoRenderer();

  MediaStream? _localStream;
  MediaStream? _remoteStream;
  RTCPeerConnection? _peerConnection;

  bool _isInitialized = false;
  bool _hasLocalStream = false;
  bool _hasRemoteStream = false;

  IceCandidateCallback? onLocalCandidate;
  RemoteStreamCallback? onRemoteStream;
  VoidCallback? onConnectionConnected;

  bool get isInitialized => _isInitialized;
  bool get hasLocalStream => _hasLocalStream;
  bool get hasRemoteStream => _hasRemoteStream;
  MediaStream? get localStream => _localStream;
  MediaStream? get remoteStream => _remoteStream;

  static const Map<String, dynamic> _rtcConfiguration = {
    'iceServers': [
      {'urls': 'stun:stun.l.google.com:19302'},
      {'urls': 'stun:stun1.l.google.com:19302'},
    ],
    'sdpSemantics': 'unified-plan',
  };

  static const Map<String, dynamic> _connectionConstraints = {
    'mandatory': {
      'OfferToReceiveAudio': true,
      'OfferToReceiveVideo': true,
    },
    'optional': [],
  };

  /// Initialize renderers and capture hardware media
  Future<void> initialize({required bool isVideoCall}) async {
    if (_isInitialized) return;

    try {
      try {
        await localRenderer.initialize().timeout(const Duration(milliseconds: 300));
        await remoteRenderer.initialize().timeout(const Duration(milliseconds: 300));
      } catch (e) {
        debugPrint('[WebRTCService] Renderer init skipped or timed out: $e');
      }

      // Attempt to acquire real hardware media stream
      final Map<String, dynamic> mediaConstraints = {
        'audio': true,
        'video': isVideoCall
            ? {
                'facingMode': 'user',
                'width': {'ideal': 1280},
                'height': {'ideal': 720},
              }
            : false,
      };

      try {
        _localStream = await navigator.mediaDevices
            .getUserMedia(mediaConstraints)
            .timeout(const Duration(milliseconds: 600));
        localRenderer.srcObject = _localStream;
        _hasLocalStream = true;
      } catch (err) {
        debugPrint('[WebRTCService] Warning: Could not access hardware camera/mic: $err');
        _hasLocalStream = false;
      }

      // Create RTCPeerConnection
      try {
        _peerConnection = await createPeerConnection(
          _rtcConfiguration,
          _connectionConstraints,
        ).timeout(const Duration(milliseconds: 500));

        _peerConnection?.onIceCandidate = (RTCIceCandidate candidate) {
          if (candidate.candidate != null && candidate.candidate!.isNotEmpty) {
            onLocalCandidate?.call(candidate);
          }
        };

        _peerConnection?.onConnectionState = (RTCPeerConnectionState state) {
          debugPrint('[WebRTCService] Connection state: $state');
          if (state == RTCPeerConnectionState.RTCPeerConnectionStateConnected) {
            onConnectionConnected?.call();
          }
        };

        _peerConnection?.onTrack = (RTCTrackEvent event) {
          debugPrint('[WebRTCService] Received remote track: ${event.track.kind}');
          if (event.streams.isNotEmpty) {
            _remoteStream = event.streams[0];
            remoteRenderer.srcObject = _remoteStream;
            _hasRemoteStream = true;
            onRemoteStream?.call(_remoteStream!);
          }
        };

        // Add local media tracks if hardware capture succeeded
        if (_localStream != null && _peerConnection != null) {
          for (final track in _localStream!.getTracks()) {
            await _peerConnection!.addTrack(track, _localStream!);
          }
        }
      } catch (e) {
        debugPrint('[WebRTCService] PeerConnection init skipped or timed out: $e');
      }

      _isInitialized = true;
    } catch (e) {
      debugPrint('[WebRTCService] Initialization error: $e');
    }
  }

  /// Caller: Create WebRTC Offer SDP
  Future<String?> createOffer({required bool isVideoCall}) async {
    if (_peerConnection == null) return null;
    try {
      final constraints = <String, dynamic>{
        'mandatory': {
          'OfferToReceiveAudio': true,
          'OfferToReceiveVideo': isVideoCall,
        },
        'optional': [],
      };

      final offer = await _peerConnection!.createOffer(constraints);
      await _peerConnection!.setLocalDescription(offer);
      return offer.sdp;
    } catch (e) {
      debugPrint('[WebRTCService] createOffer error: $e');
      return null;
    }
  }

  /// Callee: Set remote Offer SDP and create WebRTC Answer SDP
  Future<String?> createAnswer(String remoteOfferSdp) async {
    if (_peerConnection == null) return null;
    try {
      final remoteDesc = RTCSessionDescription(remoteOfferSdp, 'offer');
      await _peerConnection!.setRemoteDescription(remoteDesc);

      final answer = await _peerConnection!.createAnswer();
      await _peerConnection!.setLocalDescription(answer);
      return answer.sdp;
    } catch (e) {
      debugPrint('[WebRTCService] createAnswer error: $e');
      return null;
    }
  }

  /// Caller: Set remote Answer SDP
  Future<void> handleRemoteAnswer(String remoteAnswerSdp) async {
    if (_peerConnection == null) return;
    try {
      final remoteDesc = RTCSessionDescription(remoteAnswerSdp, 'answer');
      await _peerConnection!.setRemoteDescription(remoteDesc);
    } catch (e) {
      debugPrint('[WebRTCService] handleRemoteAnswer error: $e');
    }
  }

  /// Add incoming ICE Candidate from remote peer
  Future<void> addIceCandidate({
    required String candidate,
    String? sdpMid,
    int? sdpMLineIndex,
  }) async {
    if (_peerConnection == null) return;
    try {
      final iceCandidate = RTCIceCandidate(
        candidate,
        sdpMid,
        sdpMLineIndex ?? 0,
      );
      await _peerConnection!.addCandidate(iceCandidate);
    } catch (e) {
      debugPrint('[WebRTCService] addIceCandidate error: $e');
    }
  }

  /// Toggle microphone mute
  void setMicrophoneMute(bool isMuted) {
    if (_localStream != null) {
      for (final track in _localStream!.getAudioTracks()) {
        track.enabled = !isMuted;
      }
    }
  }

  /// Toggle video camera on/off
  void setCameraEnabled(bool isEnabled) {
    if (_localStream != null) {
      for (final track in _localStream!.getVideoTracks()) {
        track.enabled = isEnabled;
      }
    }
  }

  /// Flip between front and back camera
  Future<bool> switchCamera() async {
    if (_localStream != null) {
      final videoTracks = _localStream!.getVideoTracks();
      if (videoTracks.isNotEmpty) {
        return await Helper.switchCamera(videoTracks.first);
      }
    }
    return false;
  }

  /// Clean teardown of WebRTC media, peer connections, and renderers
  Future<void> dispose() async {
    try {
      if (_localStream != null) {
        for (final track in _localStream!.getTracks()) {
          track.stop();
        }
        await _localStream!.dispose();
        _localStream = null;
      }
      _hasLocalStream = false;

      if (_remoteStream != null) {
        for (final track in _remoteStream!.getTracks()) {
          track.stop();
        }
        await _remoteStream!.dispose();
        _remoteStream = null;
      }
      _hasRemoteStream = false;

      await _peerConnection?.close();
      _peerConnection = null;

      localRenderer.srcObject = null;
      remoteRenderer.srcObject = null;

      await localRenderer.dispose();
      await remoteRenderer.dispose();

      _isInitialized = false;
    } catch (e) {
      debugPrint('[WebRTCService] Dispose error: $e');
    }
  }
}

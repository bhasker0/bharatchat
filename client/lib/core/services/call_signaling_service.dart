import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../network/api_client.dart';
import 'notification_service.dart';
import 'sound_service.dart';
import '../../features/calls/incoming_call_dialog.dart';
import '../../features/calls/active_call_screen.dart';

final callSignalingServiceProvider = Provider<CallSignalingService>((ref) {
  return CallSignalingService();
});

class CallSignalingService {
  static final CallSignalingService _instance = CallSignalingService._internal();
  factory CallSignalingService() => _instance;
  CallSignalingService._internal();

  GlobalKey<NavigatorState>? navigatorKey;

  Timer? _pollTimer;
  String? _currentDid;
  String? _currentName;
  ApiClient? _apiClient;

  bool _isInActiveCall = false;
  bool _isIncomingDialogVisible = false;
  String? _activeIncomingCallId;
  BuildContext? _activeDialogContext;

  final Set<String> _handledCallIds = {};

  void init(GlobalKey<NavigatorState> navKey) {
    navigatorKey = navKey;
  }

  void setIsInCall(bool inCall) {
    _isInActiveCall = inCall;
  }

  bool get isInActiveCall => _isInActiveCall;
  bool get isIncomingDialogVisible => _isIncomingDialogVisible;
  String? get currentDid => _currentDid;
  String? get currentName => _currentName;

  void startListening({
    required String myDid,
    required String myName,
    required ApiClient apiClient,
  }) {
    if (_currentDid == myDid && _pollTimer != null && _pollTimer!.isActive) {
      return;
    }

    _currentDid = myDid;
    _currentName = myName;
    _apiClient = apiClient;

    _pollTimer?.cancel();
    _pollTimer = Timer.periodic(const Duration(milliseconds: 1000), (_) {
      _pollIncomingSignals();
    });
  }

  void stopListening() {
    _pollTimer?.cancel();
    _pollTimer = null;
    if (_isIncomingDialogVisible) {
      dismissIncomingCall(sendReject: false);
    }
    _currentDid = null;
    _currentName = null;
  }

  Future<void> _pollIncomingSignals() async {
    final did = _currentDid;
    final client = _apiClient;
    if (did == null || client == null) return;

    try {
      final signals = await client.getCallSignals(did: did);

      // 1. If currently showing an incoming call dialog, verify it wasn't cancelled by caller
      if (_isIncomingDialogVisible && _activeIncomingCallId != null) {
        final currentCall = signals.firstWhere(
          (s) => s['call_id'] == _activeIncomingCallId,
          orElse: () => <String, dynamic>{},
        );

        if (currentCall.isNotEmpty) {
          final status = (currentCall['status'] as String? ?? '').toUpperCase();
          if (status == 'ENDED' || status == 'REJECTED') {
            // Caller cancelled
            await dismissIncomingCall(sendReject: false);
            return;
          }
        }
      }

      // 2. If not busy, check for any incoming OFFER signals
      if (!_isInActiveCall && !_isIncomingDialogVisible) {
        for (final signal in signals) {
          final recipientDid = signal['recipient_did'] as String? ?? '';
          final signalType = (signal['signal_type'] as String? ?? '').toUpperCase();
          final status = (signal['status'] as String? ?? '').toUpperCase();
          final callId = signal['call_id'] as String? ?? '';

          if (recipientDid == did &&
              signalType == 'OFFER' &&
              status == 'RINGING' &&
              !_handledCallIds.contains(callId)) {
            final callerDid = signal['caller_did'] as String? ?? 'did:bharat:unknown';
            final callerName = signal['caller_name'] as String? ?? callerDid;
            final callType = signal['call_type'] as String? ?? 'AUDIO';
            final isVideo = callType.toUpperCase() == 'VIDEO';

            _triggerIncomingCall(
              callId: callId,
              callerDid: callerDid,
              callerName: callerName,
              isVideoCall: isVideo,
            );
            break;
          }
        }
      }
    } catch (e) {
      debugPrint('[CallSignaling] Error polling incoming signals: $e');
    }
  }

  void _triggerIncomingCall({
    required String callId,
    required String callerDid,
    required String callerName,
    required bool isVideoCall,
  }) {
    final navState = navigatorKey?.currentState;
    if (navState == null) {
      debugPrint('[CallSignaling] Cannot show incoming call dialog: navigatorState is null');
      return;
    }

    _isIncomingDialogVisible = true;
    _activeIncomingCallId = callId;

    // Start ringing sound & show local notification
    SoundService().startRinging();
    NotificationService().showCallNotification(
      callerName: callerName,
      callType: isVideoCall ? 'Video' : 'Audio',
    );

    navState.push(
      PageRouteBuilder(
        opaque: false,
        barrierDismissible: false,
        barrierColor: Colors.black.withOpacity(0.85),
        pageBuilder: (dialogCtx, _, __) {
          _activeDialogContext = dialogCtx;
          return IncomingCallDialog(
            callId: callId,
            callerDid: callerDid,
            callerName: callerName,
            isVideoCall: isVideoCall,
            onDecline: () async {
              if (_handledCallIds.length > 100) _handledCallIds.clear();
              _handledCallIds.add(callId);
              await dismissIncomingCall(
                sendReject: true,
                callId: callId,
                callerDid: callerDid,
                callType: isVideoCall ? 'VIDEO' : 'AUDIO',
              );
            },
            onAccept: () async {
              if (_handledCallIds.length > 100) _handledCallIds.clear();
              _handledCallIds.add(callId);
              await acceptIncomingCall(
                callId: callId,
                callerDid: callerDid,
                callerName: callerName,
                isVideoCall: isVideoCall,
              );
            },
          );
        },
      ),
    );
  }

  Future<void> acceptIncomingCall({
    required String callId,
    required String callerDid,
    required String callerName,
    required bool isVideoCall,
  }) async {
    _isIncomingDialogVisible = false;
    _activeIncomingCallId = null;

    await SoundService().stopRinging();
    await NotificationService().cancelCallNotification();

    if (_activeDialogContext != null) {
      final nav = Navigator.maybeOf(_activeDialogContext!);
      if (nav != null && nav.canPop()) {
        nav.pop();
      }
      _activeDialogContext = null;
    }

    // Navigate to ActiveCallScreen which initializes WebRTC, generates SDP Answer, and sends ANSWER signal
    final nav = navigatorKey?.currentState;
    if (nav != null) {
      nav.push(
        MaterialPageRoute(
          builder: (_) => ActiveCallScreen(
            callId: callId,
            recipientDid: callerDid,
            recipientName: callerName,
            isVideoCall: isVideoCall,
            isIncoming: true,
          ),
        ),
      );
    }
  }

  Future<void> dismissIncomingCall({
    bool sendReject = false,
    String? callId,
    String? callerDid,
    String? callType,
  }) async {
    _isIncomingDialogVisible = false;
    final cid = callId ?? _activeIncomingCallId;
    _activeIncomingCallId = null;

    await SoundService().stopRinging();
    await NotificationService().cancelCallNotification();

    if (_activeDialogContext != null) {
      final nav = Navigator.maybeOf(_activeDialogContext!);
      if (nav != null && nav.canPop()) {
        nav.pop();
      }
      _activeDialogContext = null;
    }

    if (sendReject && cid != null && callerDid != null && _apiClient != null) {
      final myDid = _currentDid ?? 'did:bharat:unknown';
      await _apiClient!.sendCallSignal(
        callId: cid,
        callerDid: callerDid,
        recipientDid: myDid,
        signalType: 'REJECT',
        callType: callType ?? 'AUDIO',
      );

      await _apiClient!.recordCallLog(
        callerDid: callerDid,
        recipientDid: myDid,
        callType: callType ?? 'AUDIO',
        status: 'DECLINED',
        durationSeconds: 0,
      );
    }
  }
}

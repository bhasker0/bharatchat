import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:bharatchat_client/core/network/api_client.dart';
import 'package:bharatchat_client/core/services/call_signaling_service.dart';
import 'package:bharatchat_client/core/services/sound_service.dart';
import 'package:bharatchat_client/features/calls/incoming_call_dialog.dart';
import 'package:bharatchat_client/features/calls/active_call_screen.dart';
import 'package:bharatchat_client/features/auth/auth_state.dart';

class MockCallApiClient extends ApiClient {
  final List<Map<String, dynamic>> sentSignals = [];
  final List<Map<String, dynamic>> mockIncomingSignals = [];

  @override
  Future<bool> sendCallSignal({
    required String callerDid,
    required String recipientDid,
    required String signalType,
    required String callType,
    String? callId,
    String? callerName,
    int? timestamp,
    String? sdp,
    String? candidate,
    String? sdpMid,
    int? sdpMLineIndex,
  }) async {
    sentSignals.add({
      'call_id': callId,
      'caller_did': callerDid,
      'caller_name': callerName,
      'recipient_did': recipientDid,
      'signal_type': signalType,
      'call_type': callType,
      if (sdp != null) 'sdp': sdp,
      if (candidate != null) 'candidate': candidate,
    });
    return true;
  }

  @override
  Future<List<Map<String, dynamic>>> getCallSignals({
    required String did,
    String? callId,
    int? since,
  }) async {
    return mockIncomingSignals;
  }

  @override
  Future<Map<String, dynamic>> recordCallLog({
    required String callerDid,
    required String recipientDid,
    String callType = 'VIDEO',
    String status = 'COMPLETED',
    int durationSeconds = 0,
  }) async {
    return {'success': true};
  }
}

class MockAuthNotifier extends AuthNotifier {
  final SovereignUser mockUser;
  MockAuthNotifier(this.mockUser) : super(MockCallApiClient()) {
    state = AuthState(
      isAuthenticated: true,
      user: mockUser,
    );
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Call Signaling & UI Flow Tests', () {
    setUp(() {
      SoundService().isSoundEnabled = false;
      CallSignalingService().setIsInCall(false);
      CallSignalingService().stopListening();
    });

    testWidgets('IncomingCallDialog displays caller info and handles Decline action', (tester) async {
      bool declined = false;
      bool accepted = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: IncomingCallDialog(
              callId: 'call_test_123',
              callerDid: 'did:bharat:9876543210',
              callerName: 'Rajesh Kumar',
              isVideoCall: false,
              onDecline: () {
                declined = true;
              },
              onAccept: () {
                accepted = true;
              },
            ),
          ),
        ),
      );

      // Verify UI elements
      expect(find.text('Rajesh Kumar'), findsOneWidget);
      expect(find.text('did:bharat:9876543210'), findsOneWidget);
      expect(find.text('INCOMING AUDIO CALL'), findsOneWidget);
      expect(find.text('Ringing...'), findsOneWidget);
      expect(find.text('Decline'), findsOneWidget);
      expect(find.text('Accept'), findsOneWidget);

      // Tap Decline
      await tester.tap(find.text('Decline'));
      await tester.pump();

      expect(declined, isTrue);
      expect(accepted, isFalse);
    });

    testWidgets('IncomingCallDialog displays video badge and handles Accept action', (tester) async {
      bool declined = false;
      bool accepted = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: IncomingCallDialog(
              callId: 'call_test_456',
              callerDid: 'did:bharat:1122334455',
              callerName: 'Priya Patel',
              isVideoCall: true,
              onDecline: () {
                declined = true;
              },
              onAccept: () {
                accepted = true;
              },
            ),
          ),
        ),
      );

      expect(find.text('Priya Patel'), findsOneWidget);
      expect(find.text('INCOMING VIDEO CALL'), findsOneWidget);

      // Tap Accept
      await tester.tap(find.text('Accept'));
      await tester.pump();

      expect(accepted, isTrue);
      expect(declined, isFalse);
    });

    testWidgets('ActiveCallScreen starts in Ringing state on outgoing call without auto-connecting', (tester) async {
      final mockClient = MockCallApiClient();
      final testUser = SovereignUser(
        did: 'did:bharat:caller_001',
        name: 'Test Caller',
        phone: '+919999999999',
        jurisdiction: 'Bharat, IN',
        isSimBound: true,
        registeredAt: DateTime.now(),
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            apiClientProvider.overrideWithValue(mockClient),
            authProvider.overrideWith((ref) => MockAuthNotifier(testUser)),
          ],
          child: const MaterialApp(
            home: ActiveCallScreen(
              recipientDid: 'did:bharat:recipient_002',
              recipientName: 'Recipient User',
              isVideoCall: false,
              isIncoming: false,
            ),
          ),
        ),
      );

      await tester.pump(const Duration(milliseconds: 200));

      // Verify header and Ringing... state (NOT auto-connected)
      expect(find.text('Recipient User'), findsOneWidget);
      expect(find.text('Ringing...'), findsOneWidget);

      // Verify OFFER signal sent
      expect(mockClient.sentSignals.length, 1);
      expect(mockClient.sentSignals.first['signal_type'], 'OFFER');
      expect(mockClient.sentSignals.first['caller_did'], 'did:bharat:caller_001');
      expect(mockClient.sentSignals.first['recipient_did'], 'did:bharat:recipient_002');

      // Fast forward 3 seconds: verify it STILL stays on Ringing... and DOES NOT auto-connect!
      await tester.pump(const Duration(seconds: 3));
      expect(find.text('Ringing...'), findsOneWidget);

      // Tap End Call
      await tester.tap(find.byIcon(Icons.call_end));
      await tester.pumpAndSettle();

      // Verify CANCEL signal sent
      expect(mockClient.sentSignals.any((s) => s['signal_type'] == 'CANCEL'), isTrue);
    });

    test('CallSignalingService manages in-call state and listeners cleanly', () {
      final service = CallSignalingService();
      expect(service.isInActiveCall, isFalse);

      service.setIsInCall(true);
      expect(service.isInActiveCall, isTrue);

      service.setIsInCall(false);
      expect(service.isInActiveCall, isFalse);

      service.stopListening();
      expect(service.currentDid, isNull);
    });
  });
}

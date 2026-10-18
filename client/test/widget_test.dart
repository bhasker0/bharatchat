import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:bharatchat_client/main.dart';
import 'package:bharatchat_client/features/auth/auth_screen.dart';
import 'package:bharatchat_client/features/auth/auth_state.dart';
import 'package:bharatchat_client/core/network/api_client.dart';
import 'package:bharatchat_client/features/chat/chat_screen.dart';
import 'package:bharatchat_client/features/updates/updates_screen.dart';
import 'package:bharatchat_client/features/calls/group_call_screen.dart';
import 'package:bharatchat_client/features/commerce/catalog_screen.dart';
import 'package:bharatchat_client/features/compliance/compliance_screen.dart';

class MockTestApiClient extends ApiClient {
  @override
  Future<Map<String, dynamic>> registerIdentity({
    required String did,
    required String vkHex,
    required int timestamp,
  }) async {
    return {
      'success': true,
      'data': {
        'did': did,
        'vk_hex': vkHex,
        'timestamp': timestamp,
      },
    };
  }

  @override
  Future<Map<String, dynamic>> getIdentity(String did) async {
    return {
      'success': true,
      'data': {
        'did': did,
        'status': 'ACTIVE',
      },
    };
  }

  @override
  Future<List<Map<String, dynamic>>> getRegisteredIdentities() async {
    return [
      {'did': 'did:bharat:9876543210', 'status': 'ACTIVE'},
    ];
  }

  @override
  Future<Map<String, dynamic>> sendMessage({
    required String senderDid,
    required String recipientDid,
    required String text,
  }) async {
    return {'success': true};
  }

  @override
  Future<List<Map<String, dynamic>>> getMessages({
    required String did,
    int since = 0,
  }) async {
    return [];
  }
}

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('BharatChat Sovereign Client Widget & Flow Tests', () {
    testWidgets('App root boots to AuthScreen with Sign In and Registration tabs',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            apiClientProvider.overrideWithValue(MockTestApiClient()),
          ],
          child: const BharatChatApp(),
        ),
      );
      await tester.pumpAndSettle();

      // 1. Verify AuthScreen is loaded by default
      expect(find.byType(AuthScreen), findsOneWidget);
      expect(find.text('BharatChat Sovereign'), findsOneWidget);
      expect(find.text("India's End-to-End Encrypted Sovereign Platform"), findsOneWidget);
      expect(find.text('Sign In'), findsOneWidget);
      expect(find.text('Create Account'), findsOneWidget);

      // 2. Switch to Create Account tab
      await tester.tap(find.text('Create Account'));
      await tester.pumpAndSettle();
      expect(find.text('Full Name / Business Display Name'), findsOneWidget);
      expect(find.text('Mobile Number (+91 India)'), findsOneWidget);
      expect(find.text('Register Sovereign DID'), findsOneWidget);

      // 3. Switch back to Sign In
      await tester.tap(find.text('Sign In'));
      await tester.pumpAndSettle();
      expect(find.text('Verify & Sign In'), findsOneWidget);

      // 4. Test 1-Tap Demo sign in transitions to clean authenticated chat screen
      final demoButton = find.text('Instant Demo Access (Ramesh Patel • Surat)');
      expect(demoButton, findsOneWidget);
      await tester.tap(demoButton);
      await tester.pumpAndSettle();

      // 5. Verify transition to RootNavigationScreen with clean empty chat state
      expect(find.byType(ChatScreen), findsOneWidget);
      expect(find.widgetWithText(AppBar, 'Chats'), findsOneWidget);
      expect(find.text('No Conversations Yet'), findsOneWidget);
      expect(find.text('Start New Chat'), findsOneWidget);
    });

    testWidgets('Registration form completes, user searches ledger and starts 1-on-1 chat',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            apiClientProvider.overrideWithValue(MockTestApiClient()),
          ],
          child: const BharatChatApp(),
        ),
      );
      await tester.pumpAndSettle();

      // 1. Switch to Create Account
      await tester.tap(find.text('Create Account'));
      await tester.pumpAndSettle();

      // 2. Enter name and phone
      final textFields = find.byType(TextField);
      await tester.enterText(textFields.at(0), 'Bhaskar Savaliya');
      await tester.enterText(textFields.at(1), '9898012345');
      await tester.pumpAndSettle();

      // 3. Tap Register
      await tester.tap(find.text('Register Sovereign DID'));
      await tester.pumpAndSettle();

      // 4. Verify app navigates to main screen with empty state
      expect(find.byType(ChatScreen), findsOneWidget);
      expect(find.widgetWithText(AppBar, 'Chats'), findsOneWidget);
      expect(find.text('No Conversations Yet'), findsOneWidget);

      // 5. Tap Start New Chat to open user discovery sheet
      await tester.tap(find.text('Start New Chat'));
      await tester.pumpAndSettle();
      expect(find.text('Find User & Start Chat'), findsOneWidget);

      // 6. Enter recipient phone number to search ledger
      final searchField = find.byType(TextField).first;
      await tester.enterText(searchField, '9876543210');
      await tester.tap(find.text('Search'));
      await tester.pumpAndSettle();

      // 7. Verify ledger found the account
      expect(find.text('Active on Sovereign Local Ledger'), findsOneWidget);

      // 8. Tap Chat to open 1-on-1 chat screen
      await tester.tap(find.text('Chat').first);
      await tester.pumpAndSettle();

      // 9. Verify ChatDetailScreen is open
      expect(find.byType(ChatDetailScreen), findsOneWidget);
      expect(find.text('Messages are end-to-end encrypted. Rule 4(2) Originator Envelope is sealed to multi-sig judicial key.'), findsOneWidget);

      // 10. Type and send a message
      final msgField = find.byType(TextField).first;
      await tester.enterText(msgField, 'Namaste Sovereign Bharat!');
      await tester.tap(find.byIcon(Icons.send));
      await tester.pumpAndSettle();

      expect(find.text('Namaste Sovereign Bharat!'), findsOneWidget);
    });

    testWidgets('Navigation bar smoothly transitions between sovereign modules',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      // Boot with pre-authenticated state
      final authNotifier = AuthNotifier();
      authNotifier.setDemoUser('Ramesh Patel', '9876543210');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            apiClientProvider.overrideWithValue(MockTestApiClient()),
            authProvider.overrideWith((ref) => authNotifier),
          ],
          child: const BharatChatApp(),
        ),
      );
      await tester.pumpAndSettle();

      // 1. Switch to Updates tab (Status & Channels)
      await tester.tap(find.text('Updates'));
      await tester.pumpAndSettle();
      expect(find.byType(UpdatesScreen), findsOneWidget);
      expect(find.text('Status'), findsOneWidget);
      expect(find.text('My status'), findsOneWidget);
      expect(find.text('Tap to add status update'), findsOneWidget);
      expect(find.text('Channels'), findsOneWidget);
      expect(find.text('PIB India Fact Check'), findsOneWidget);
      expect(find.text('Forwarded many times (>= 5 hops)'), findsOneWidget);

      // 2. Switch to Calls tab
      await tester.tap(find.text('Calls (32)'));
      await tester.pumpAndSettle();
      expect(find.byType(GroupCallScreen), findsOneWidget);
      expect(find.text('No Recent Calls'), findsOneWidget);
      expect(find.text('Start 32-Party SFrame Call'), findsOneWidget);

      // Tap start call to enter SFrame call room
      await tester.tap(find.text('Start 32-Party SFrame Call'));
      await tester.pumpAndSettle();
      expect(find.text('Surat Textiles Group Call (6/32)'), findsOneWidget);
      expect(find.text('SFrame E2EE Active • Epoch #12'), findsOneWidget);

      // 3. Switch to Catalog tab
      await tester.tap(find.text('Catalog'));
      await tester.pumpAndSettle();
      expect(find.byType(CatalogScreen), findsOneWidget);
      expect(find.text('Pure Kanjivaram Silk Saree'), findsOneWidget);

      // 4. Switch to DPDP Compliance tab
      await tester.tap(find.text('DPDP / Law'));
      await tester.pumpAndSettle();
      expect(find.byType(ComplianceScreen), findsOneWidget);
      expect(find.text('DPDP & Legal Compliance Dashboard'), findsOneWidget);
      expect(find.text('Ramesh Patel'), findsOneWidget);
      expect(find.text('Sign Out of Sovereign Session'), findsOneWidget);

      // 5. Test Sign Out returns to AuthScreen
      await tester.tap(find.text('Sign Out of Sovereign Session'));
      await tester.pumpAndSettle();
      expect(find.byType(AuthScreen), findsOneWidget);
    });

    testWidgets('DPDP Section 12 Right-to-Erasure modal triggers crypto-shredding state',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 5000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: ComplianceScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // Find and tap the Crypto-Shred button
      final shredButtonFinder = find.text('Execute DPDP Section 12 Crypto-Shredding');
      expect(shredButtonFinder, findsOneWidget);

      await tester.tap(shredButtonFinder);
      await tester.pumpAndSettle();

      // Confirm dialog appeared
      expect(find.text('DPDP Act 2023 Section 12 Right to Erasure'), findsOneWidget);
      expect(find.text('Confirm Crypto-Shred'), findsOneWidget);

      // Tap confirm button
      await tester.tap(find.text('Confirm Crypto-Shred'));
      await tester.pumpAndSettle();

      // State should update to purged
      expect(find.text('REVOKED_AND_PURGED'), findsOneWidget);
    });
  });
}

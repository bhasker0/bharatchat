import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:bharatchat_client/main.dart';
import 'package:bharatchat_client/features/chat/chat_screen.dart';
import 'package:bharatchat_client/features/calls/group_call_screen.dart';
import 'package:bharatchat_client/features/commerce/catalog_screen.dart';
import 'package:bharatchat_client/features/compliance/compliance_screen.dart';

void main() {
  group('BharatChat Sovereign Client Widget & Flow Tests', () {
    testWidgets('App root boots with ProviderScope and BottomNavigationBar',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: BharatChatApp(),
        ),
      );
      await tester.pumpAndSettle();

      // Verify bottom navigation items exist
      expect(find.text('Chats'), findsOneWidget);
      expect(find.text('Calls (32)'), findsOneWidget);
      expect(find.text('Catalog'), findsOneWidget);
      expect(find.text('DPDP / Law'), findsOneWidget);

      // Verify default tab is ChatScreen
      expect(find.byType(ChatScreen), findsOneWidget);
      expect(find.text('Surat Heritage Silk'), findsOneWidget);
      expect(find.text('Verified Merchant • E2EE Encrypted'), findsOneWidget);
    });

    testWidgets('Navigation bar smoothly transitions between sovereign modules',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: BharatChatApp(),
        ),
      );
      await tester.pumpAndSettle();

      // 1. Switch to Calls tab
      await tester.tap(find.text('Calls (32)'));
      await tester.pumpAndSettle();
      expect(find.byType(GroupCallScreen), findsOneWidget);
      expect(find.text('Surat Textiles Group Call (6/32)'), findsOneWidget);
      expect(find.text('SFrame E2EE Active • Epoch #12'), findsOneWidget);

      // 2. Switch to Catalog tab
      await tester.tap(find.text('Catalog'));
      await tester.pumpAndSettle();
      expect(find.byType(CatalogScreen), findsOneWidget);
      expect(find.text('Pure Kanjivaram Silk Saree'), findsOneWidget);

      // 3. Switch to DPDP Compliance tab
      await tester.tap(find.text('DPDP / Law'));
      await tester.pumpAndSettle();
      expect(find.byType(ComplianceScreen), findsOneWidget);
      expect(find.text('DPDP & Legal Compliance Dashboard'), findsOneWidget);
    });

    testWidgets('DPDP Section 12 Right-to-Erasure modal triggers crypto-shredding state',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: ComplianceScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // Find the Crypto-Shred button
      final shredButtonFinder = find.widgetWithText(ElevatedButton, 'Execute DPDP Section 12 Crypto-Shredding');
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

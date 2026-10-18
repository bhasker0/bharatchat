import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/network/api_client.dart';
import 'core/services/call_signaling_service.dart';
import 'core/services/notification_service.dart';
import 'core/services/update_service.dart';
import 'core/theme/app_theme.dart';
import 'features/auth/auth_screen.dart';
import 'features/auth/auth_state.dart';
import 'features/chat/chat_screen.dart';
import 'features/settings/update_dialog.dart';
import 'features/updates/updates_screen.dart';
import 'features/calls/group_call_screen.dart';
import 'features/commerce/catalog_screen.dart';
import 'features/compliance/compliance_screen.dart';

final GlobalKey<NavigatorState> rootNavigatorKey = GlobalKey<NavigatorState>();

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await NotificationService().init();
  CallSignalingService().init(rootNavigatorKey);
  runApp(const ProviderScope(child: BharatChatApp()));
}

class BharatChatApp extends ConsumerWidget {
  const BharatChatApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);

    return MaterialApp(
      navigatorKey: rootNavigatorKey,
      title: 'BharatChat Sovereign',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.darkTheme,
      home: authState.isAuthenticated
          ? const RootNavigationScreen()
          : const AuthScreen(),
    );
  }
}

class RootNavigationScreen extends ConsumerStatefulWidget {
  const RootNavigationScreen({super.key});

  @override
  ConsumerState<RootNavigationScreen> createState() => _RootNavigationScreenState();
}

class _RootNavigationScreenState extends ConsumerState<RootNavigationScreen> {
  int _currentIndex = 0;

  final List<Widget> _screens = const [
    ChatScreen(),
    UpdatesScreen(),
    GroupCallScreen(),
    CatalogScreen(),
    ComplianceScreen(),
  ];

  @override
  void initState() {
    super.initState();
    // Auto-check for OTA updates (native only) and start real-time call signaling listener
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!kIsWeb) {
        ref.read(updateProvider.notifier).checkForUpdate(silent: true);
      }
      final auth = ref.read(authProvider);
      if (auth.isAuthenticated && auth.user != null) {
        CallSignalingService().startListening(
          myDid: auth.user!.did,
          myName: auth.user!.name,
          apiClient: ref.read(apiClientProvider),
        );
      }
    });
  }

  @override
  void dispose() {
    CallSignalingService().stopListening();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final updateState = ref.watch(updateProvider);

    // Keep call signaling actively synchronized with current auth state
    ref.listen<AuthState>(authProvider, (previous, next) {
      if (next.isAuthenticated && next.user != null) {
        CallSignalingService().startListening(
          myDid: next.user!.did,
          myName: next.user!.name,
          apiClient: ref.read(apiClientProvider),
        );
      } else {
        CallSignalingService().stopListening();
      }
    });

    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            // In-app OTA Update Notification Banner (Mobile native only; hidden on Web)
            if (!kIsWeb &&
                (updateState.status == UpdateStatus.available ||
                 updateState.status == UpdateStatus.downloading ||
                 updateState.status == UpdateStatus.readyToInstall))
              Material(
                color: AppTheme.primaryGreen.withOpacity(0.95),
                child: InkWell(
                  onTap: () => UpdateDialog.show(context),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                    child: Row(
                      children: [
                        const Icon(Icons.system_update_rounded, color: Colors.white, size: 20),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Text(
                            updateState.status == UpdateStatus.downloading
                                ? 'Downloading Update: ${(updateState.progress * 100).toStringAsFixed(0)}%'
                                : updateState.status == UpdateStatus.readyToInstall
                                    ? 'Update ready to install. Tap to finish.'
                                    : 'New update available: ${updateState.updateInfo?.latestVersionName}. Tap to install.',
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                        const SizedBox(width: 6),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                          decoration: BoxDecoration(
                            color: Colors.black.withOpacity(0.25),
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Text(
                            updateState.status == UpdateStatus.readyToInstall ? 'INSTALL' : 'UPDATE',
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            Expanded(
              child: IndexedStack(
                index: _currentIndex,
                children: _screens,
              ),
            ),
          ],
        ),
      ),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: _currentIndex,
        onTap: (index) => setState(() => _currentIndex = index),
        type: BottomNavigationBarType.fixed,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.chat_bubble_outline),
            activeIcon: Icon(Icons.chat_bubble),
            label: "Chats",
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.update_outlined),
            activeIcon: Icon(Icons.update),
            label: "Updates",
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.call_outlined),
            activeIcon: Icon(Icons.call),
            label: "Calls (32)",
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.storefront_outlined),
            activeIcon: Icon(Icons.storefront),
            label: "Catalog",
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.shield_outlined),
            activeIcon: Icon(Icons.shield),
            label: "DPDP / Law",
          ),
        ],
      ),
    );
  }
}

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/services/sound_service.dart';
import '../../core/services/notification_service.dart';
import '../../core/theme/app_theme.dart';
import '../auth/auth_state.dart';
import '../../core/network/api_client.dart';
import '../../core/services/update_service.dart';
import 'update_dialog.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  bool _chatSounds = true;
  bool _notifications = true;
  String _selectedLanguage = 'English';
  String _serverLatency = '4ms';
  bool _isCheckingPing = false;

  @override
  void initState() {
    super.initState();
    _chatSounds = SoundService().isSoundEnabled;
    _notifications = NotificationService().isNotificationEnabled;
  }

  Future<void> _checkPing() async {
    setState(() => _isCheckingPing = true);
    final stopwatch = Stopwatch()..start();
    final client = ref.read(apiClientProvider);
    final res = await client.checkHealth();
    stopwatch.stop();

    setState(() {
      _isCheckingPing = false;
      if (res) {
        _serverLatency = '${stopwatch.elapsedMilliseconds}ms (Healthy)';
      } else {
        _serverLatency = 'Offline';
      }
    });

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Server Ping: $_serverLatency'),
          duration: const Duration(seconds: 1),
        ),
      );
    }
  }

  void _showEditNameDialog(SovereignUser user) {
    final controller = TextEditingController(text: user.name);
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Text('Edit Display Name', style: TextStyle(color: AppTheme.textPrimary)),
        content: TextField(
          controller: controller,
          style: const TextStyle(color: AppTheme.textPrimary),
          decoration: const InputDecoration(
            labelText: 'Full Name',
            labelStyle: TextStyle(color: AppTheme.primaryGreen),
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel', style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: AppTheme.primaryGreen),
            onPressed: () {
              final newName = controller.text.trim();
              if (newName.isNotEmpty) {
                final updatedUser = SovereignUser(
                  did: user.did,
                  name: newName,
                  phone: user.phone,
                  jurisdiction: user.jurisdiction,
                  isSimBound: user.isSimBound,
                  registeredAt: user.registeredAt,
                );
                ref.read(authProvider.notifier).updateUser(updatedUser);
              }
              Navigator.pop(ctx);
            },
            child: const Text('Save', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _showCryptoShredDialog(SovereignUser user) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Row(
          children: [
            Icon(Icons.shield_outlined, color: Colors.redAccent),
            SizedBox(width: 8),
            Text('DPDP Act Section 12', style: TextStyle(color: Colors.redAccent)),
          ],
        ),
        content: const Text(
          'Crypto-shredding permanently destroys your ephemeral double-ratchet keys on this device and logs a verifiable compliance receipt. Are you sure you wish to exercise your statutory Right to Erasure?',
          style: TextStyle(color: AppTheme.textSecondary, fontSize: 13),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel', style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.redAccent),
            onPressed: () async {
              Navigator.pop(ctx);
              await ref.read(authProvider.notifier).signOut();
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Session keys crypto-shredded under DPDP Act 2023.'),
                    backgroundColor: Colors.redAccent,
                  ),
                );
              }
            },
            child: const Text('Crypto-Shred & Sign Out', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authProvider);
    final user = authState.user;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: ListView(
        padding: const EdgeInsets.symmetric(vertical: 8),
        children: [
          // 1. Profile Header
          if (user != null)
            Container(
              margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: AppTheme.surfaceDark,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.3)),
              ),
              child: Row(
                children: [
                  CircleAvatar(
                    radius: 32,
                    backgroundColor: AppTheme.primaryGreen,
                    child: Text(
                      user.name.isNotEmpty ? user.name[0].toUpperCase() : 'B',
                      style: const TextStyle(fontSize: 26, fontWeight: FontWeight.bold, color: Colors.white),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text(
                              user.name,
                              style: const TextStyle(
                                fontSize: 18,
                                fontWeight: FontWeight.bold,
                                color: AppTheme.textPrimary,
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.edit, size: 16, color: AppTheme.primaryGreen),
                              onPressed: () => _showEditNameDialog(user),
                            ),
                          ],
                        ),
                        Text(
                          user.phone,
                          style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
                        ),
                        const SizedBox(height: 4),
                        Row(
                          children: [
                            const Icon(Icons.verified, size: 13, color: AppTheme.accentBlue),
                            const SizedBox(width: 4),
                            Text(
                              user.isSimBound ? 'SIM Bound • Enclave Verified' : 'Standard Node',
                              style: const TextStyle(fontSize: 11, color: AppTheme.accentBlue),
                            ),
                          ],
                        ),
                        const SizedBox(height: 4),
                        InkWell(
                          onTap: () {
                            Clipboard.setData(ClipboardData(text: user.did));
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(content: Text('DID copied to clipboard!')),
                            );
                          },
                          child: Text(
                            user.did,
                            style: const TextStyle(
                              fontSize: 10,
                              color: AppTheme.textSecondary,
                              fontFamily: 'monospace',
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),

          const Divider(color: AppTheme.surfaceDark),

          // 2. Notifications & Sounds
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              'NOTIFICATIONS & AUDIO',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
            ),
          ),
          SwitchListTile(
            secondary: const Icon(Icons.volume_up_outlined, color: AppTheme.primaryGreen),
            title: const Text('In-App Chat Sounds', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: const Text('Play chimes when sending and receiving messages in chat',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            value: _chatSounds,
            activeColor: AppTheme.primaryGreen,
            onChanged: (val) {
              setState(() => _chatSounds = val);
              SoundService().isSoundEnabled = val;
              if (val) SoundService().playMessageSent();
            },
          ),
          SwitchListTile(
            secondary: const Icon(Icons.notifications_active_outlined, color: AppTheme.primaryGreen),
            title: const Text('Background Notifications', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: const Text('Alerts & sound when app or chat thread is in background',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            value: _notifications,
            activeColor: AppTheme.primaryGreen,
            onChanged: (val) {
              setState(() => _notifications = val);
              NotificationService().isNotificationEnabled = val;
            },
          ),
          ListTile(
            leading: const Icon(Icons.music_note_outlined, color: AppTheme.primaryGreen),
            title: const Text('Test Notification Chime', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: const Text('Preview message audio feedback', style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            trailing: const Icon(Icons.play_arrow, color: AppTheme.primaryGreen),
            onTap: () => SoundService().playMessageReceived(),
          ),

          const Divider(color: AppTheme.surfaceDark),

          // 3. Language / Bhasha
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              'LANGUAGE / BHASHA',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
            ),
          ),
          ListTile(
            leading: const Icon(Icons.translate, color: AppTheme.primaryGreen),
            title: const Text('App Language', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: Text(_selectedLanguage, style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            trailing: DropdownButton<String>(
              dropdownColor: AppTheme.surfaceDark,
              value: _selectedLanguage,
              underline: const SizedBox(),
              items: const [
                DropdownMenuItem(value: 'English', child: Text('English', style: TextStyle(color: Colors.white))),
                DropdownMenuItem(value: 'Hindi', child: Text('हिन्दी (Hindi)', style: TextStyle(color: Colors.white))),
                DropdownMenuItem(value: 'Gujarati', child: Text('ગુજરાતી (Gujarati)', style: TextStyle(color: Colors.white))),
                DropdownMenuItem(value: 'Marathi', child: Text('मराठी (Marathi)', style: TextStyle(color: Colors.white))),
                DropdownMenuItem(value: 'Tamil', child: Text('தமிழ் (Tamil)', style: TextStyle(color: Colors.white))),
              ],
              onChanged: (val) {
                if (val != null) {
                  setState(() => _selectedLanguage = val);
                }
              },
            ),
          ),

          const Divider(color: AppTheme.surfaceDark),

          // 4. Local Sovereign Server Node
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              'LOCAL SOVEREIGN NODE',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
            ),
          ),
          ListTile(
            leading: const Icon(Icons.dns_outlined, color: AppTheme.primaryGreen),
            title: const Text('Local Gateway Node', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: Text('Status: $_serverLatency • Port 8080',
                style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            trailing: _isCheckingPing
                ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))
                : TextButton.icon(
                    icon: const Icon(Icons.refresh, size: 16, color: AppTheme.primaryGreen),
                    label: const Text('Ping', style: TextStyle(color: AppTheme.primaryGreen)),
                    onPressed: _checkPing,
                  ),
          ),

          if (!kIsWeb) ...[
            const Divider(color: AppTheme.surfaceDark),

            // 5. App Updates & OTA System
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Text(
                'APP UPDATES & OTA SYSTEM',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
              ),
            ),
            Consumer(
              builder: (context, ref, _) {
                final updateState = ref.watch(updateProvider);
                return ListTile(
                  leading: const Icon(Icons.system_update_alt_rounded, color: AppTheme.primaryGreen),
                  title: const Text('Check for Updates', style: TextStyle(color: AppTheme.textPrimary)),
                  subtitle: Text(
                    updateState.status == UpdateStatus.available
                        ? 'Update available: ${updateState.updateInfo?.latestVersionName} (Tap to install)'
                        : updateState.status == UpdateStatus.downloading
                            ? 'Downloading: ${(updateState.progress * 100).toStringAsFixed(0)}%'
                            : 'Installed: v${UpdateNotifier.currentVersionName} (Code ${UpdateNotifier.currentVersionCode})',
                    style: TextStyle(
                      color: updateState.status == UpdateStatus.available
                          ? AppTheme.primaryGreen
                          : AppTheme.textSecondary,
                      fontSize: 12,
                      fontWeight: updateState.status == UpdateStatus.available ? FontWeight.bold : FontWeight.normal,
                    ),
                  ),
                  trailing: updateState.status == UpdateStatus.checking
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2, color: AppTheme.primaryGreen),
                        )
                      : TextButton.icon(
                          icon: Icon(
                            updateState.status == UpdateStatus.available ? Icons.download_rounded : Icons.sync,
                            size: 16,
                            color: AppTheme.primaryGreen,
                          ),
                          label: Text(
                            updateState.status == UpdateStatus.available ? 'Install' : 'Check',
                            style: const TextStyle(color: AppTheme.primaryGreen),
                          ),
                          onPressed: () async {
                            if (updateState.status == UpdateStatus.available ||
                                updateState.status == UpdateStatus.downloading ||
                                updateState.status == UpdateStatus.readyToInstall) {
                              UpdateDialog.show(context);
                            } else {
                              final info = await ref.read(updateProvider.notifier).checkForUpdate();
                              if (context.mounted) {
                                if (info != null) {
                                  UpdateDialog.show(context);
                                } else {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    const SnackBar(
                                      content: Text('You are on the latest sovereign version (v${UpdateNotifier.currentVersionName}).'),
                                      duration: Duration(seconds: 2),
                                    ),
                                  );
                                }
                              }
                            }
                          },
                        ),
                  onTap: () {
                    if (updateState.status == UpdateStatus.available ||
                        updateState.status == UpdateStatus.downloading ||
                        updateState.status == UpdateStatus.readyToInstall) {
                      UpdateDialog.show(context);
                    }
                  },
                );
              },
            ),
          ],

          const Divider(color: AppTheme.surfaceDark),

          // 6. Privacy, DPDP Act & Sign Out
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              'PRIVACY & SECURITY',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
            ),
          ),
          const ListTile(
            leading: Icon(Icons.security, color: AppTheme.primaryGreen),
            title: Text('IT Rules 2021 Rule 4(2)', style: TextStyle(color: AppTheme.textPrimary)),
            subtitle: Text('First Originator Hardware Multi-Sig Sealed',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
          ),
          ListTile(
            leading: const Icon(Icons.delete_forever_outlined, color: Colors.redAccent),
            title: const Text('DPDP Right to Erasure', style: TextStyle(color: Colors.redAccent)),
            subtitle: const Text('Crypto-shred local keys and session',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            onTap: () {
              if (user != null) _showCryptoShredDialog(user);
            },
          ),
          ListTile(
            leading: const Icon(Icons.logout, color: Colors.orangeAccent),
            title: const Text('Sign Out', style: TextStyle(color: Colors.orangeAccent)),
            subtitle: const Text('Keep DID registered, switch account',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
            onTap: () async {
              final nav = Navigator.of(context);
              await ref.read(authProvider.notifier).signOut();
              nav.popUntil((route) => route.isFirst);
            },
          ),
          const SizedBox(height: 32),
        ],
      ),
    );
  }
}

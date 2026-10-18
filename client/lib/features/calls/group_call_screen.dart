import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/network/api_client.dart';
import '../../core/theme/app_theme.dart';
import '../auth/auth_state.dart';
import 'active_call_screen.dart';

class GroupCallScreen extends ConsumerStatefulWidget {
  const GroupCallScreen({super.key});

  @override
  ConsumerState<GroupCallScreen> createState() => _GroupCallScreenState();
}

class _GroupCallScreenState extends ConsumerState<GroupCallScreen> {
  bool _isInCall = false;
  DateTime _callStartTime = DateTime.now();
  List<Map<String, dynamic>> _callLogs = [];
  bool _isLoading = false;

  final participants = const [
    {"name": "You (Host)", "isSpeaking": true, "videoOn": true},
    {"name": "Surat Silk Munim", "isSpeaking": false, "videoOn": true},
    {"name": "Design Head Priya", "isSpeaking": false, "videoOn": false},
    {"name": "Master Karigar Suresh", "isSpeaking": true, "videoOn": true},
    {"name": "Logistics Hub Surat", "isSpeaking": false, "videoOn": false},
    {"name": "Compliance Legal", "isSpeaking": false, "videoOn": false},
  ];

  @override
  void initState() {
    super.initState();
    _loadCallLogs();
  }

  Future<void> _loadCallLogs() async {
    final user = ref.read(authProvider).user;
    final did = user?.did ?? 'did:bharat:unknown';
    setState(() => _isLoading = true);
    final logs = await ref.read(apiClientProvider).getCallLogs(did: did);
    if (mounted) {
      setState(() {
        _callLogs = logs;
        _isLoading = false;
      });
    }
  }

  void _startGroupCall() {
    setState(() {
      _isInCall = true;
      _callStartTime = DateTime.now();
    });
  }

  Future<void> _launchOneOnOneCall(String recipientDid, String recipientName, {bool isVideo = false}) async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ActiveCallScreen(
          recipientDid: recipientDid,
          recipientName: recipientName,
          isVideoCall: isVideo,
        ),
      ),
    );
    if (mounted) {
      await _loadCallLogs();
    }
  }

  void _showNewCallSheet() {
    final didController = TextEditingController();
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) => Padding(
        padding: EdgeInsets.only(
          left: 20,
          right: 20,
          top: 20,
          bottom: MediaQuery.of(ctx).viewInsets.bottom + 24,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.add_call, color: AppTheme.primaryGreen),
                const SizedBox(width: 10),
                const Text(
                  'Start Encrypted Call',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close, color: AppTheme.textSecondary),
                  onPressed: () => Navigator.pop(ctx),
                ),
              ],
            ),
            const SizedBox(height: 16),
            TextField(
              controller: didController,
              style: const TextStyle(color: AppTheme.textPrimary),
              decoration: InputDecoration(
                labelText: 'Recipient DID or Phone (+91...)',
                hintText: 'e.g. +91 98765 43210',
                prefixIcon: const Icon(Icons.person_search, color: AppTheme.primaryGreen),
                filled: true,
                fillColor: AppTheme.darkBackground,
                border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton.icon(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppTheme.surfaceDark,
                      foregroundColor: AppTheme.primaryGreen,
                      side: const BorderSide(color: AppTheme.primaryGreen),
                      padding: const EdgeInsets.symmetric(vertical: 12),
                    ),
                    icon: const Icon(Icons.call),
                    label: const Text('Voice Call'),
                    onPressed: () {
                      final target = didController.text.trim();
                      if (target.isEmpty) return;
                      final did = target.startsWith('did:') ? target : 'did:bharat:${target.replaceAll(RegExp(r'\D'), '')}';
                      Navigator.pop(ctx);
                      _launchOneOnOneCall(did, target, isVideo: false);
                    },
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: ElevatedButton.icon(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppTheme.primaryGreen,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(vertical: 12),
                    ),
                    icon: const Icon(Icons.videocam),
                    label: const Text('Video Call'),
                    onPressed: () {
                      final target = didController.text.trim();
                      if (target.isEmpty) return;
                      final did = target.startsWith('did:') ? target : 'did:bharat:${target.replaceAll(RegExp(r'\D'), '')}';
                      Navigator.pop(ctx);
                      _launchOneOnOneCall(did, target, isVideo: true);
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            const Divider(color: Colors.white12),
            const SizedBox(height: 8),
            ListTile(
              contentPadding: EdgeInsets.zero,
              leading: Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: AppTheme.primaryGreen.withOpacity(0.15),
                  shape: BoxShape.circle,
                ),
                child: const Icon(Icons.groups, color: AppTheme.primaryGreen),
              ),
              title: const Text(
                '32-Party SFrame Textile Room',
                style: TextStyle(fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
              ),
              subtitle: const Text(
                'Multi-party post-quantum E2EE group conference',
                style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
              ),
              trailing: const Icon(Icons.arrow_forward_ios, size: 16, color: AppTheme.textSecondary),
              onTap: () {
                Navigator.pop(ctx);
                _startGroupCall();
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _endCall() async {
    final duration = DateTime.now().difference(_callStartTime).inSeconds;
    final user = ref.read(authProvider).user;
    final did = user?.did ?? 'did:bharat:unknown';

    setState(() => _isInCall = false);

    await ref.read(apiClientProvider).recordCallLog(
          callerDid: did,
          recipientDid: 'did:bharat:sframe_group_32',
          callType: 'GROUP_SFRAME',
          status: 'COMPLETED',
          durationSeconds: duration > 0 ? duration : 5,
        );

    await _loadCallLogs();
  }

  String _formatCallDuration(int seconds) {
    if (seconds < 60) return '${seconds}s';
    final mins = seconds ~/ 60;
    final secs = seconds % 60;
    return '${mins}m ${secs}s';
  }

  String _formatTimestamp(int ts) {
    if (ts <= 0) return 'Just now';
    final dt = DateTime.fromMillisecondsSinceEpoch(ts * 1000);
    return '${dt.day}/${dt.month} ${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}';
  }

  @override
  Widget build(BuildContext context) {
    if (!_isInCall) {
      return Scaffold(
        appBar: AppBar(
          title: const Text("Calls"),
          actions: [
            IconButton(
              icon: const Icon(Icons.refresh),
              tooltip: "Refresh Call Logs",
              onPressed: _loadCallLogs,
            ),
            IconButton(
              icon: const Icon(Icons.add_call),
              tooltip: "Start New Call",
              onPressed: _showNewCallSheet,
            ),
          ],
        ),
        body: _isLoading
            ? const Center(child: CircularProgressIndicator())
            : _callLogs.isEmpty
                ? _buildEmptyState()
                : _buildCallList(),
        floatingActionButton: FloatingActionButton(
          heroTag: 'call_fab',
          backgroundColor: AppTheme.primaryGreen,
          foregroundColor: Colors.white,
          tooltip: "Start Call",
          onPressed: _showNewCallSheet,
          child: const Icon(Icons.add_call),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text("Surat Textiles Group Call (6/32)", style: TextStyle(fontSize: 16)),
            Text(
              "SFrame E2EE Active • Epoch #12",
              style: TextStyle(fontSize: 11, color: AppTheme.primaryGreen),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.person_add_outlined),
            onPressed: () {},
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: GridView.builder(
              padding: const EdgeInsets.all(8),
              gridDelegate: const DynamicGridDelegate(),
              itemCount: participants.length,
              itemBuilder: (context, index) {
                final p = participants[index];
                final isSpeaking = p["isSpeaking"] as bool;
                final videoOn = p["videoOn"] as bool;

                return Container(
                  margin: const EdgeInsets.all(4),
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceDark,
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(
                      color: isSpeaking ? AppTheme.primaryGreen : Colors.transparent,
                      width: 2,
                    ),
                  ),
                  child: Stack(
                    children: [
                      Center(
                        child: videoOn
                            ? const Icon(Icons.person, size: 48, color: AppTheme.textSecondary)
                            : const Icon(Icons.videocam_off_outlined, size: 40, color: Colors.grey),
                      ),
                      Positioned(
                        bottom: 8,
                        left: 8,
                        right: 8,
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.black.withOpacity(0.6),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Row(
                            children: [
                              Icon(
                                isSpeaking ? Icons.mic : Icons.mic_off,
                                size: 12,
                                color: isSpeaking ? AppTheme.primaryGreen : Colors.red,
                              ),
                              const SizedBox(width: 4),
                              Expanded(
                                child: Text(
                                  p["name"] as String,
                                  style: const TextStyle(fontSize: 11, color: Colors.white),
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
          // Call Controls Bar
          Container(
            padding: const EdgeInsets.symmetric(vertical: 16),
            color: AppTheme.surfaceDark,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildCircleButton(Icons.mic, "Mute", Colors.grey[800]!, () {}),
                _buildCircleButton(Icons.videocam, "Video", Colors.grey[800]!, () {}),
                _buildCircleButton(Icons.screen_share, "Share", Colors.grey[800]!, () {}),
                _buildCircleButton(Icons.call_end, "End", Colors.red, _endCall),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 96,
              height: 96,
              decoration: BoxDecoration(
                color: AppTheme.surfaceDark,
                shape: BoxShape.circle,
                border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.3), width: 2),
              ),
              child: const Icon(
                Icons.call_outlined,
                size: 48,
                color: AppTheme.primaryGreen,
              ),
            ),
            const SizedBox(height: 20),
            const Text(
              "No Recent Calls",
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              "BharatChat calls use post-quantum SFrame end-to-end encryption supporting up to 32 participants simultaneously. No voice or video data is recorded or stored on any server.",
              style: TextStyle(
                fontSize: 13,
                color: AppTheme.textSecondary,
                height: 1.4,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                OutlinedButton.icon(
                  style: OutlinedButton.styleFrom(
                    foregroundColor: AppTheme.primaryGreen,
                    side: const BorderSide(color: AppTheme.primaryGreen),
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                  ),
                  icon: const Icon(Icons.call),
                  label: const Text(
                    "1-on-1 Call",
                    style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                  ),
                  onPressed: _showNewCallSheet,
                ),
                const SizedBox(width: 12),
                ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryGreen,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                  ),
                  icon: const Icon(Icons.video_call),
                  label: const Text(
                    "Start 32-Party SFrame Call",
                    style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                  ),
                  onPressed: _startGroupCall,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCallList() {
    final user = ref.watch(authProvider).user;
    final myDid = user?.did ?? '';

    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _callLogs.length,
      separatorBuilder: (_, __) => const Divider(
        color: AppTheme.surfaceDark,
        height: 1,
        indent: 72,
      ),
      itemBuilder: (context, index) {
        final log = _callLogs[index];
        final callerDid = log['caller_did'] as String? ?? '';
        final recipientDid = log['recipient_did'] as String? ?? '';
        final callType = log['call_type'] as String? ?? 'AUDIO';
        final duration = log['duration_seconds'] as int? ?? 0;
        final ts = log['timestamp'] as int? ?? 0;

        final isOutbound = callerDid == myDid;
        final partnerDid = isOutbound ? recipientDid : callerDid;
        final isGroup = partnerDid.contains('sframe_group') || callType.contains('GROUP');
        final displayName = isGroup
            ? 'Surat Textiles SFrame Group (32)'
            : partnerDid.replaceFirst('did:bharat:', '+91 ');
        final isVideo = callType.contains('VIDEO');

        return ListTile(
          onTap: () {
            if (isGroup) {
              _startGroupCall();
            } else {
              _launchOneOnOneCall(partnerDid, displayName, isVideo: isVideo);
            }
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          leading: CircleAvatar(
            radius: 24,
            backgroundColor: AppTheme.primaryGreen.withOpacity(0.2),
            child: Icon(
              isGroup ? Icons.groups_outlined : Icons.person_outline,
              color: AppTheme.primaryGreen,
              size: 24,
            ),
          ),
          title: Text(
            displayName,
            style: const TextStyle(
              fontWeight: FontWeight.bold,
              fontSize: 15,
              color: AppTheme.textPrimary,
            ),
          ),
          subtitle: Row(
            children: [
              Icon(
                isOutbound ? Icons.call_made : Icons.call_received,
                size: 14,
                color: isOutbound ? AppTheme.primaryGreen : AppTheme.accentBlue,
              ),
              const SizedBox(width: 4),
              Text(
                '${_formatTimestamp(ts)} • ${_formatCallDuration(duration)}',
                style: const TextStyle(
                  fontSize: 12,
                  color: AppTheme.textSecondary,
                ),
              ),
            ],
          ),
          trailing: IconButton(
            icon: Icon(
              isVideo || isGroup ? Icons.videocam : Icons.call,
              color: AppTheme.primaryGreen,
            ),
            tooltip: 'Call Back',
            onPressed: () {
              if (isGroup) {
                _startGroupCall();
              } else {
                _launchOneOnOneCall(partnerDid, displayName, isVideo: isVideo);
              }
            },
          ),
        );
      },
    );
  }

  Widget _buildCircleButton(
    IconData icon,
    String label,
    Color bg,
    VoidCallback onTap,
  ) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        CircleAvatar(
          radius: 24,
          backgroundColor: bg,
          child: IconButton(
            icon: Icon(icon, color: Colors.white, size: 20),
            onPressed: onTap,
          ),
        ),
        const SizedBox(height: 4),
        Text(label, style: const TextStyle(fontSize: 10, color: AppTheme.textSecondary)),
      ],
    );
  }
}

class DynamicGridDelegate extends SliverGridDelegateWithFixedCrossAxisCount {
  const DynamicGridDelegate()
      : super(
          crossAxisCount: 2,
          crossAxisSpacing: 8,
          mainAxisSpacing: 8,
          childAspectRatio: 1.1,
        );
}

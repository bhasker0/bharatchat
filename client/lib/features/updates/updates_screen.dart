import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';

enum StatusAudienceOption {
  myContacts,
  myContactsExcept,
  onlyShareWith,
}

class StatusState {
  final StatusAudienceOption audience;
  final List<String> myStories;
  final Map<String, bool> followedChannels;

  const StatusState({
    required this.audience,
    required this.myStories,
    required this.followedChannels,
  });

  StatusState copyWith({
    StatusAudienceOption? audience,
    List<String>? myStories,
    Map<String, bool>? followedChannels,
  }) {
    return StatusState(
      audience: audience ?? this.audience,
      myStories: myStories ?? this.myStories,
      followedChannels: followedChannels ?? this.followedChannels,
    );
  }
}

class StatusNotifier extends Notifier<StatusState> {
  @override
  StatusState build() {
    return const StatusState(
      audience: StatusAudienceOption.myContacts,
      myStories: [
        "Celebrating Republic Day in New Delhi! 🇮🇳 (Expires in 22h)",
      ],
      followedChannels: {
        "chan_pib_india": true,
        "chan_isro": true,
      },
    );
  }

  void setAudience(StatusAudienceOption option) {
    state = state.copyWith(audience: option);
  }

  void addStory(String caption) {
    state = state.copyWith(myStories: [...state.myStories, caption]);
  }

  void toggleChannelFollow(String channelId) {
    final updated = Map<String, bool>.from(state.followedChannels);
    updated[channelId] = !(updated[channelId] ?? false);
    state = state.copyWith(followedChannels: updated);
  }
}

final statusProvider = NotifierProvider<StatusNotifier, StatusState>(StatusNotifier.new);

class UpdatesScreen extends ConsumerWidget {
  const UpdatesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statusState = ref.watch(statusProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Updates'),
        actions: [
          IconButton(
            icon: const Icon(Icons.privacy_tip_outlined),
            tooltip: 'Status Privacy',
            onPressed: () => _showPrivacySheet(context, ref),
          ),
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {},
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.symmetric(vertical: 12.0),
        children: [
          // ------------------------------------------------------------------
          // STATUS SECTION
          // ------------------------------------------------------------------
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'Status',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                    color: AppTheme.textPrimary,
                  ),
                ),
                TextButton.icon(
                  icon: const Icon(Icons.lock_clock, size: 16, color: AppTheme.primaryGreen),
                  label: const Text('24h Auto-Shred', style: TextStyle(color: AppTheme.primaryGreen)),
                  onPressed: () => _showPrivacySheet(context, ref),
                ),
              ],
            ),
          ),
          ListTile(
            leading: Stack(
              children: [
                Container(
                  padding: const EdgeInsets.all(2.5),
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    border: Border.all(color: AppTheme.primaryGreen, width: 2.5),
                  ),
                  child: const CircleAvatar(
                    radius: 24,
                    backgroundColor: AppTheme.surfaceDark,
                    child: Icon(Icons.person, color: AppTheme.textPrimary),
                  ),
                ),
                Positioned(
                  bottom: 0,
                  right: 0,
                  child: Container(
                    decoration: const BoxDecoration(
                      shape: BoxShape.circle,
                      color: AppTheme.primaryGreen,
                    ),
                    padding: const EdgeInsets.all(2.0),
                    child: const Icon(Icons.add, size: 16, color: Colors.black),
                  ),
                ),
              ],
            ),
            title: const Text('My status', style: TextStyle(fontWeight: FontWeight.w600)),
            subtitle: Text(
              statusState.myStories.isNotEmpty
                  ? statusState.myStories.last
                  : 'Tap to add status update',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: AppTheme.textSecondary),
            ),
            trailing: IconButton(
              icon: const Icon(Icons.camera_alt, color: AppTheme.primaryGreen),
              onPressed: () {
                ref.read(statusProvider.notifier).addStory("New Sovereign Update! 🇮🇳 (Expires in 24h)");
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Status posted: 24h ephemeral timer active')),
                );
              },
            ),
          ),
          const Divider(color: AppTheme.surfaceDark),

          // Recent updates
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
            child: Text(
              'Recent updates',
              style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppTheme.textSecondary),
            ),
          ),
          ListTile(
            leading: Container(
              padding: const EdgeInsets.all(2.5),
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                border: Border.all(color: AppTheme.primaryGreen, width: 2.5),
              ),
              child: const CircleAvatar(
                radius: 24,
                backgroundColor: AppTheme.surfaceDark,
                child: Text('🇮🇳', style: TextStyle(fontSize: 22)),
              ),
            ),
            title: const Text('Priya Sharma', style: TextStyle(fontWeight: FontWeight.w600)),
            subtitle: const Text('15 minutes ago • DPDP Notice Accepted', style: TextStyle(color: AppTheme.textSecondary)),
            onTap: () {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Viewing Priya\'s E2EE Status (Mutual Contacts Only)')),
              );
            },
          ),
          ListTile(
            leading: Container(
              padding: const EdgeInsets.all(2.5),
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                border: Border.all(color: AppTheme.primaryGreen, width: 2.5),
              ),
              child: const CircleAvatar(
                radius: 24,
                backgroundColor: AppTheme.surfaceDark,
                child: Text('🪔', style: TextStyle(fontSize: 22)),
              ),
            ),
            title: const Text('Varanasi Weavers Collective', style: TextStyle(fontWeight: FontWeight.w600)),
            subtitle: const Text('Today, 2:15 PM • Handloom Catalog Live', style: TextStyle(color: AppTheme.textSecondary)),
            onTap: () {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Viewing Weavers Status: Silk Weaving Showcase')),
              );
            },
          ),
          const SizedBox(height: 16),

          // ------------------------------------------------------------------
          // CHANNELS SECTION
          // ------------------------------------------------------------------
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Channels',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                    color: AppTheme.textPrimary,
                  ),
                ),
                Text(
                  'Find channels',
                  style: TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryGreen,
                  ),
                ),
              ],
            ),
          ),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16.0, vertical: 4.0),
            child: Text(
              'Stay updated on topics you care about. Verified channels deliver tamper-evident broadcasts.',
              style: TextStyle(color: AppTheme.textSecondary, fontSize: 13),
            ),
          ),

          // Channel 1: PIB Fact Check
          _buildChannelTile(
            context,
            ref,
            channelId: 'chan_pib_india',
            name: 'PIB India Fact Check',
            description: 'Official Government of India Fact Checking Unit. Verified by MeitY.',
            badge: 'Government Official',
            isFollowed: statusState.followedChannels['chan_pib_india'] ?? false,
            recentPost: 'Beware of fake SMS regarding lottery schemes. Always verify via gov.in portals.',
            reactions: '👍 1.2k • 🇮🇳 3.4k',
          ),

          // Channel 2: ISRO Space Dispatch
          _buildChannelTile(
            context,
            ref,
            channelId: 'chan_isro',
            name: 'ISRO Space Dispatch',
            description: 'Official announcements from Indian Space Research Organisation.',
            badge: 'National Scientific Agency',
            isFollowed: statusState.followedChannels['chan_isro'] ?? false,
            recentPost: 'Gaganyaan crew module qualification test successfully completed.',
            reactions: '🚀 5.8k • 🇮🇳 12.1k',
          ),

          const SizedBox(height: 24),

          // Viral Misinformation Friction Demo Tile
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 16.0),
            padding: const EdgeInsets.all(14.0),
            decoration: BoxDecoration(
              color: AppTheme.surfaceDark,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.amber.withOpacity(0.4)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Row(
                  children: [
                    Icon(Icons.forward, color: Colors.amber, size: 20),
                    Icon(Icons.forward, color: Colors.amber, size: 20),
                    SizedBox(width: 8),
                    Text(
                      'Forwarded many times (>= 5 hops)',
                      style: TextStyle(fontWeight: FontWeight.bold, color: Colors.amber),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                const Text(
                  'Under Indian Anti-Disinformation Mandates, this message has reached the viral friction threshold. It can only be forwarded to 1 chat at a time.',
                  style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
                ),
                const SizedBox(height: 12),
                ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryGreen,
                    foregroundColor: Colors.black,
                  ),
                  icon: const Icon(Icons.share, size: 16),
                  label: const Text('Attempt Multi-Chat Forward'),
                  onPressed: () {
                    showDialog(
                      context: context,
                      builder: (ctx) => AlertDialog(
                        backgroundColor: AppTheme.surfaceDark,
                        title: const Row(
                          children: [
                            Icon(Icons.warning_amber_rounded, color: Colors.amber),
                            SizedBox(width: 8),
                            Text('Forwarding Restricted'),
                          ],
                        ),
                        content: const Text(
                          'Frequently forwarded messages can only be forwarded to 1 chat at a time to prevent viral misinformation spread.\n\nStatute: IT Rules 2021 & Misinformation Friction Standard.',
                        ),
                        actions: [
                          TextButton(
                            child: const Text('OK', style: TextStyle(color: AppTheme.primaryGreen)),
                            onPressed: () => Navigator.pop(ctx),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ],
            ),
          ),
          const SizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildChannelTile(
    BuildContext context,
    WidgetRef ref, {
    required String channelId,
    required String name,
    required String description,
    required String badge,
    required bool isFollowed,
    required String recentPost,
    required String reactions,
  }) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
      padding: const EdgeInsets.all(12.0),
      decoration: BoxDecoration(
        color: AppTheme.surfaceDark,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.white10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const CircleAvatar(
                radius: 20,
                backgroundColor: AppTheme.primaryGreen,
                child: Icon(Icons.campaign, color: Colors.black),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(name, style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 15)),
                        const SizedBox(width: 6),
                        const Icon(Icons.verified, size: 16, color: AppTheme.primaryGreen),
                      ],
                    ),
                    Text(badge, style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
                  ],
                ),
              ),
              OutlinedButton(
                style: OutlinedButton.styleFrom(
                  side: BorderSide(color: isFollowed ? AppTheme.textSecondary : AppTheme.primaryGreen),
                  foregroundColor: isFollowed ? AppTheme.textSecondary : AppTheme.primaryGreen,
                ),
                child: Text(isFollowed ? 'Following' : 'Follow'),
                onPressed: () => ref.read(statusProvider.notifier).toggleChannelFollow(channelId),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.all(10.0),
            decoration: BoxDecoration(
              color: AppTheme.darkBackground,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: const [
                    Icon(Icons.verified_user, size: 14, color: AppTheme.primaryGreen),
                    SizedBox(width: 4),
                    Text(
                      'Ed25519 Tamper-Evident Signed Post',
                      style: TextStyle(fontSize: 11, color: AppTheme.primaryGreen, fontWeight: FontWeight.w600),
                    ),
                  ],
                ),
                const SizedBox(height: 6),
                Text(recentPost, style: const TextStyle(fontSize: 13, color: AppTheme.textPrimary)),
                const SizedBox(height: 8),
                Text(reactions, style: const TextStyle(fontSize: 12, color: AppTheme.textSecondary)),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showPrivacySheet(BuildContext context, WidgetRef ref) {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        final currentAudience = ref.watch(statusProvider).audience;
        return Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 24.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Status privacy',
                style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
              ),
              const SizedBox(height: 8),
              const Text(
                'Who can see my status updates. Changes will affect new updates only.',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 13),
              ),
              const SizedBox(height: 16),
              RadioListTile<StatusAudienceOption>(
                title: const Text('My contacts'),
                subtitle: const Text('Share with all mutual contacts'),
                value: StatusAudienceOption.myContacts,
                groupValue: currentAudience,
                activeColor: AppTheme.primaryGreen,
                onChanged: (val) {
                  if (val != null) {
                    ref.read(statusProvider.notifier).setAudience(val);
                    Navigator.pop(ctx);
                  }
                },
              ),
              RadioListTile<StatusAudienceOption>(
                title: const Text('My contacts except...'),
                subtitle: const Text('Hide status from selected contacts'),
                value: StatusAudienceOption.myContactsExcept,
                groupValue: currentAudience,
                activeColor: AppTheme.primaryGreen,
                onChanged: (val) {
                  if (val != null) {
                    ref.read(statusProvider.notifier).setAudience(val);
                    Navigator.pop(ctx);
                  }
                },
              ),
              RadioListTile<StatusAudienceOption>(
                title: const Text('Only share with...'),
                subtitle: const Text('Share strictly with whitelist'),
                value: StatusAudienceOption.onlyShareWith,
                groupValue: currentAudience,
                activeColor: AppTheme.primaryGreen,
                onChanged: (val) {
                  if (val != null) {
                    ref.read(statusProvider.notifier).setAudience(val);
                    Navigator.pop(ctx);
                  }
                },
              ),
              const SizedBox(height: 12),
            ],
          ),
        );
      },
    );
  }
}

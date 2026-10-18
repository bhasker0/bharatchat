import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';

enum StatusAudienceOption {
  myContacts,
  myContactsExcept,
  onlyShareWith,
}

class StoryModel {
  final String authorName;
  final String avatarText;
  final String caption;
  final DateTime timestamp;
  final List<Color> gradient;

  const StoryModel({
    required this.authorName,
    required this.avatarText,
    required this.caption,
    required this.timestamp,
    required this.gradient,
  });
}

class StatusState {
  final StatusAudienceOption audience;
  final List<StoryModel> myStories;
  final Map<String, bool> followedChannels;

  const StatusState({
    required this.audience,
    required this.myStories,
    required this.followedChannels,
  });

  StatusState copyWith({
    StatusAudienceOption? audience,
    List<StoryModel>? myStories,
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
      myStories: [],
      followedChannels: {
        "chan_pib_india": true,
        "chan_isro": true,
      },
    );
  }

  void setAudience(StatusAudienceOption option) {
    state = state.copyWith(audience: option);
  }

  void addStory(String caption, List<Color> gradient) {
    final story = StoryModel(
      authorName: 'My Status',
      avatarText: '👤',
      caption: caption,
      timestamp: DateTime.now(),
      gradient: gradient,
    );
    state = state.copyWith(myStories: [...state.myStories, story]);
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

  static final List<StoryModel> priyaStories = [
    StoryModel(
      authorName: 'Priya Sharma',
      avatarText: '🇮🇳',
      caption: 'Sovereign textile showcase live! Pure Banarasi & Katan Silk designs ready for dispatch. 🥻🇮🇳',
      timestamp: DateTime.now().subtract(const Duration(minutes: 15)),
      gradient: const [Color(0xFF004D40), Color(0xFF00796B)],
    ),
  ];

  static final List<StoryModel> weaversStories = [
    StoryModel(
      authorName: 'Varanasi Weavers Collective',
      avatarText: '🪔',
      caption: 'Handloom Jacquard weaving workshop stream live from Varanasi! Direct Karigar settlement enabled. 🧵✨',
      timestamp: DateTime.now().subtract(const Duration(hours: 2)),
      gradient: const [Color(0xFFE65100), Color(0xFFBF360C)],
    ),
  ];

  void _openStoryPlayer(BuildContext context, List<StoryModel> stories) {
    Navigator.of(context).push(
      PageRouteBuilder(
        opaque: false,
        pageBuilder: (_, __, ___) => StoryPlayerScreen(stories: stories),
      ),
    );
  }

  void _showCreateStatusSheet(BuildContext context, WidgetRef ref) {
    final textController = TextEditingController();
    final gradientPresets = [
      [const Color(0xFF004D40), const Color(0xFF00796B)],
      [const Color(0xFFE65100), const Color(0xFFF57C00)],
      [const Color(0xFF4A148C), const Color(0xFF7B1FA2)],
      [const Color(0xFF0D47A1), const Color(0xFF1976D2)],
      [const Color(0xFF880E4F), const Color(0xFFAD1457)],
    ];
    int selectedGradientIndex = 0;

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        return StatefulBuilder(
          builder: (modalCtx, setModalState) {
            final activeGradient = gradientPresets[selectedGradientIndex];
            return Padding(
              padding: EdgeInsets.only(
                left: 20,
                right: 20,
                top: 20,
                bottom: MediaQuery.of(modalCtx).viewInsets.bottom + 24,
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.edit_note, color: AppTheme.primaryGreen),
                      const SizedBox(width: 8),
                      const Text(
                        'Compose Ephemeral Status',
                        style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
                      ),
                      const Spacer(),
                      IconButton(
                        icon: const Icon(Icons.close, color: AppTheme.textSecondary),
                        onPressed: () => Navigator.pop(modalCtx),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Container(
                    height: 120,
                    width: double.infinity,
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        colors: activeGradient,
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      borderRadius: BorderRadius.circular(16),
                      boxShadow: [
                        BoxShadow(
                          color: activeGradient.first.withOpacity(0.4),
                          blurRadius: 12,
                          offset: const Offset(0, 4),
                        ),
                      ],
                    ),
                    child: Center(
                      child: Text(
                        textController.text.isEmpty
                            ? 'Your status preview will appear here...'
                            : textController.text,
                        style: const TextStyle(
                          fontSize: 15,
                          fontWeight: FontWeight.bold,
                          color: Colors.white,
                        ),
                        textAlign: TextAlign.center,
                        maxLines: 3,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: textController,
                    maxLines: 2,
                    style: const TextStyle(color: AppTheme.textPrimary),
                    onChanged: (_) => setModalState(() {}),
                    decoration: InputDecoration(
                      hintText: 'Type your update (auto-shreds after 24h)...',
                      filled: true,
                      fillColor: AppTheme.darkBackground,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                        borderSide: BorderSide.none,
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  const Text('Select Background Theme:', style: TextStyle(fontSize: 13, color: AppTheme.textSecondary)),
                  const SizedBox(height: 8),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: List.generate(gradientPresets.length, (index) {
                      final isSelected = selectedGradientIndex == index;
                      final g = gradientPresets[index];
                      return GestureDetector(
                        onTap: () => setModalState(() => selectedGradientIndex = index),
                        child: Container(
                          width: 40,
                          height: 40,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            gradient: LinearGradient(colors: g),
                            border: Border.all(
                              color: isSelected ? Colors.white : Colors.transparent,
                              width: 3,
                            ),
                          ),
                          child: isSelected ? const Icon(Icons.check, size: 18, color: Colors.white) : null,
                        ),
                      );
                    }),
                  ),
                  const SizedBox(height: 20),
                  SizedBox(
                    width: double.infinity,
                    child: ElevatedButton.icon(
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppTheme.primaryGreen,
                        foregroundColor: Colors.white,
                        padding: const EdgeInsets.symmetric(vertical: 14),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                      ),
                      icon: const Icon(Icons.send),
                      label: const Text(
                        'Post Sovereign Status (24h Ephemeral)',
                        style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14),
                      ),
                      onPressed: () {
                        final text = textController.text.trim();
                        if (text.isEmpty) return;
                        ref.read(statusProvider.notifier).addStory(text, activeGradient);
                        Navigator.pop(modalCtx);
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('Status posted: 24h ephemeral timer active')),
                        );
                      },
                    ),
                  ),
                ],
              ),
            );
          },
        );
      },
    );
  }

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
      floatingActionButton: FloatingActionButton(
        heroTag: 'updates_fab',
        backgroundColor: AppTheme.primaryGreen,
        foregroundColor: Colors.white,
        tooltip: 'Add Status',
        onPressed: () => _showCreateStatusSheet(context, ref),
        child: const Icon(Icons.edit),
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
            onTap: () {
              if (statusState.myStories.isNotEmpty) {
                _openStoryPlayer(context, statusState.myStories);
              } else {
                _showCreateStatusSheet(context, ref);
              }
            },
            leading: Stack(
              children: [
                Container(
                  padding: const EdgeInsets.all(2.5),
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    border: Border.all(
                      color: statusState.myStories.isNotEmpty ? AppTheme.primaryGreen : Colors.grey,
                      width: 2.5,
                    ),
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
                  ? '${statusState.myStories.length} update(s) active • Tap to view'
                  : 'Tap to add status update',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: AppTheme.textSecondary),
            ),
            trailing: IconButton(
              icon: const Icon(Icons.camera_alt, color: AppTheme.primaryGreen),
              onPressed: () => _showCreateStatusSheet(context, ref),
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
            onTap: () => _openStoryPlayer(context, priyaStories),
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
            onTap: () => _openStoryPlayer(context, weaversStories),
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
                const Row(
                  children: [
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

class StoryPlayerScreen extends StatefulWidget {
  final List<StoryModel> stories;
  final int initialIndex;

  const StoryPlayerScreen({
    super.key,
    required this.stories,
    this.initialIndex = 0,
  });

  @override
  State<StoryPlayerScreen> createState() => _StoryPlayerScreenState();
}

class _StoryPlayerScreenState extends State<StoryPlayerScreen> with SingleTickerProviderStateMixin {
  late int _currentIndex;
  late AnimationController _progressController;

  @override
  void initState() {
    super.initState();
    _currentIndex = widget.initialIndex;
    _progressController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 6),
    )..addStatusListener((status) {
        if (status == AnimationStatus.completed) {
          _nextStory();
        }
      });
    _progressController.forward();
  }

  @override
  void dispose() {
    _progressController.dispose();
    super.dispose();
  }

  void _nextStory() {
    if (_currentIndex < widget.stories.length - 1) {
      setState(() {
        _currentIndex++;
        _progressController.reset();
        _progressController.forward();
      });
    } else {
      if (mounted) Navigator.of(context).pop();
    }
  }

  void _prevStory() {
    if (_currentIndex > 0) {
      setState(() {
        _currentIndex--;
        _progressController.reset();
        _progressController.forward();
      });
    } else {
      _progressController.reset();
      _progressController.forward();
    }
  }

  String _formatTime(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return 'Today';
  }

  @override
  Widget build(BuildContext context) {
    if (widget.stories.isEmpty) {
      return const Scaffold(backgroundColor: Colors.black);
    }

    final story = widget.stories[_currentIndex];

    return Scaffold(
      backgroundColor: Colors.black,
      body: GestureDetector(
        onTapDown: (details) {
          final width = MediaQuery.of(context).size.width;
          if (details.globalPosition.dx < width * 0.35) {
            _prevStory();
          } else {
            _nextStory();
          }
        },
        child: Container(
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: story.gradient,
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
          ),
          child: SafeArea(
            child: Column(
              children: [
                // Top Progress Indicators
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
                  child: Row(
                    children: List.generate(widget.stories.length, (idx) {
                      return Expanded(
                        child: Container(
                          height: 3,
                          margin: const EdgeInsets.symmetric(horizontal: 2),
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(2),
                            color: Colors.white24,
                          ),
                          child: idx < _currentIndex
                              ? Container(color: Colors.white)
                              : idx == _currentIndex
                                  ? AnimatedBuilder(
                                      animation: _progressController,
                                      builder: (_, __) => LinearProgressIndicator(
                                        value: _progressController.value,
                                        backgroundColor: Colors.white24,
                                        valueColor: const AlwaysStoppedAnimation(Colors.white),
                                      ),
                                    )
                                  : Container(color: Colors.white24),
                        ),
                      );
                    }),
                  ),
                ),
                // Story Header: Avatar, Name, Time, Close
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 6.0),
                  child: Row(
                    children: [
                      CircleAvatar(
                        radius: 18,
                        backgroundColor: Colors.white24,
                        child: Text(story.avatarText, style: const TextStyle(fontSize: 16)),
                      ),
                      const SizedBox(width: 10),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            story.authorName,
                            style: const TextStyle(fontWeight: FontWeight.bold, color: Colors.white, fontSize: 14),
                          ),
                          Text(
                            '${_formatTime(story.timestamp)} • E2EE Ephemeral',
                            style: const TextStyle(color: Colors.white70, fontSize: 11),
                          ),
                        ],
                      ),
                      const Spacer(),
                      IconButton(
                        icon: const Icon(Icons.close, color: Colors.white),
                        onPressed: () => Navigator.of(context).pop(),
                      ),
                    ],
                  ),
                ),
                // Center Story Content
                Expanded(
                  child: Center(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 32.0),
                      child: Text(
                        story.caption,
                        style: const TextStyle(
                          fontSize: 24,
                          fontWeight: FontWeight.bold,
                          color: Colors.white,
                          height: 1.4,
                          shadows: [
                            Shadow(color: Colors.black45, blurRadius: 10, offset: Offset(0, 3)),
                          ],
                        ),
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ),
                ),
                // Bottom Quick Reaction & Reply bar
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
                  color: Colors.black38,
                  child: Row(
                    children: [
                      Expanded(
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                          decoration: BoxDecoration(
                            color: Colors.white.withOpacity(0.15),
                            borderRadius: BorderRadius.circular(24),
                          ),
                          child: Row(
                            children: [
                              const Icon(Icons.reply, size: 18, color: Colors.white70),
                              const SizedBox(width: 8),
                              Text(
                                'Reply to ${story.authorName}...',
                                style: const TextStyle(color: Colors.white70, fontSize: 13),
                              ),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),
                      _buildQuickReaction('❤️'),
                      _buildQuickReaction('👏'),
                      _buildQuickReaction('🙏'),
                      _buildQuickReaction('🔥'),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildQuickReaction(String emoji) {
    return GestureDetector(
      onTap: () {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Sent reaction $emoji to ${widget.stories[_currentIndex].authorName}'),
            duration: const Duration(seconds: 1),
          ),
        );
      },
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4.0),
        child: Text(emoji, style: const TextStyle(fontSize: 22)),
      ),
    );
  }
}


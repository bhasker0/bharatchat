import 'dart:async';
import 'package:audioplayers/audioplayers.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/network/api_client.dart';
import '../../core/services/media_service.dart';
import '../../core/services/sound_service.dart';
import '../../core/theme/app_theme.dart';
import '../auth/auth_state.dart';
import '../calls/active_call_screen.dart';
import '../settings/settings_screen.dart';
import 'chat_state.dart';

class ChatScreen extends ConsumerWidget {
  const ChatScreen({super.key});

  void _openNewChatSheet(BuildContext context, WidgetRef ref) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) => const _NewChatModal(),
    );
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatState = ref.watch(chatProvider);
    final conversations = chatState.conversations;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Chats'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            tooltip: 'Search Conversations',
            onPressed: () => _openNewChatSheet(context, ref),
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            tooltip: 'Settings',
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const SettingsScreen()),
              );
            },
          ),
        ],
      ),
      body: conversations.isEmpty
          ? _buildEmptyState(context, ref)
          : _buildConversationList(context, ref, conversations),
      floatingActionButton: FloatingActionButton(
        heroTag: 'chat_fab',
        backgroundColor: AppTheme.primaryGreen,
        foregroundColor: Colors.white,
        tooltip: 'New Chat',
        onPressed: () => _openNewChatSheet(context, ref),
        child: const Icon(Icons.chat),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context, WidgetRef ref) {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 100,
              height: 100,
              decoration: BoxDecoration(
                color: AppTheme.surfaceDark,
                shape: BoxShape.circle,
                border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.3), width: 2),
              ),
              child: const Icon(
                Icons.forum_outlined,
                size: 48,
                color: AppTheme.primaryGreen,
              ),
            ),
            const SizedBox(height: 20),
            const Text(
              "No Conversations Yet",
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              "All chats are end-to-end encrypted with zero cloud retention. Search by mobile number (+91) or select a user from the local server directory to start messaging.",
              style: TextStyle(
                fontSize: 13,
                color: AppTheme.textSecondary,
                height: 1.4,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              style: ElevatedButton.styleFrom(
                backgroundColor: AppTheme.primaryGreen,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
              ),
              icon: const Icon(Icons.person_add_alt_1),
              label: const Text(
                "Start New Chat",
                style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
              ),
              onPressed: () => _openNewChatSheet(context, ref),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConversationList(
    BuildContext context,
    WidgetRef ref,
    List<ChatConversation> conversations,
  ) {
    return ListView.separated(
      itemCount: conversations.length,
      separatorBuilder: (_, __) => const Divider(
        color: AppTheme.surfaceDark,
        height: 1,
        indent: 72,
      ),
      itemBuilder: (context, index) {
        final conv = conversations[index];
        return ListTile(
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
          leading: CircleAvatar(
            radius: 24,
            backgroundColor: AppTheme.primaryGreen.withOpacity(0.2),
            child: Text(
              conv.recipientName.isNotEmpty
                  ? conv.recipientName[0].toUpperCase()
                  : 'U',
              style: const TextStyle(
                color: AppTheme.primaryGreen,
                fontWeight: FontWeight.bold,
                fontSize: 18,
              ),
            ),
          ),
          title: Text(
            conv.recipientName,
            style: const TextStyle(
              fontWeight: FontWeight.bold,
              fontSize: 15,
              color: AppTheme.textPrimary,
            ),
          ),
          subtitle: Row(
            children: [
              const Icon(Icons.lock_outline, size: 12, color: AppTheme.accentBlue),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  conv.lastMessage,
                  style: const TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
          trailing: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                conv.lastMessageTime,
                style: const TextStyle(fontSize: 11, color: AppTheme.textSecondary),
              ),
              if (conv.unreadCount > 0) ...[
                const SizedBox(height: 4),
                Container(
                  padding: const EdgeInsets.all(5),
                  decoration: const BoxDecoration(
                    color: AppTheme.primaryGreen,
                    shape: BoxShape.circle,
                  ),
                  child: Text(
                    '${conv.unreadCount}',
                    style: const TextStyle(fontSize: 10, color: Colors.white, fontWeight: FontWeight.bold),
                  ),
                ),
              ],
            ],
          ),
          onTap: () {
            Navigator.of(context).push(
              MaterialPageRoute(
                builder: (_) => ChatDetailScreen(conversation: conv),
              ),
            );
          },
        );
      },
    );
  }
}

class _NewChatModal extends ConsumerStatefulWidget {
  const _NewChatModal();

  @override
  ConsumerState<_NewChatModal> createState() => _NewChatModalState();
}

class _NewChatModalState extends ConsumerState<_NewChatModal> {
  final _searchController = TextEditingController();
  bool _isSearching = false;
  Map<String, dynamic>? _foundIdentity;
  String? _searchError;
  late Future<List<Map<String, dynamic>>> _directoryFuture;

  @override
  void initState() {
    super.initState();
    _directoryFuture = ref.read(apiClientProvider).getRegisteredIdentities();
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _handleSearch() async {
    final query = _searchController.text.trim();
    if (query.isEmpty) return;

    final clean = query.replaceAll(RegExp(r'\D'), '');
    final did = clean.isNotEmpty
        ? 'did:bharat:${clean.length > 10 ? clean.substring(clean.length - 10) : clean}'
        : query;

    setState(() {
      _isSearching = true;
      _foundIdentity = null;
      _searchError = null;
    });

    final res = await ref.read(apiClientProvider).getIdentity(did);

    setState(() {
      _isSearching = false;
      if (res['success'] == true) {
        _foundIdentity = res['data'];
      } else {
        _searchError = res['error'] ?? 'Identity not found on local sovereign ledger.';
      }
    });
  }

  void _openChatWithDid(String did, [String? name]) {
    final displayName = name ?? did.replaceFirst('did:bharat:', '+91 ');
    final conv = ref.read(chatProvider.notifier).startConversation(
          recipientDid: did,
          recipientName: displayName,
        );

    Navigator.of(context).pop(); // close modal
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ChatDetailScreen(conversation: conv),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final currentUserDid = ref.watch(authProvider).user?.did ?? '';

    return Padding(
      padding: EdgeInsets.only(
        top: 20,
        left: 20,
        right: 20,
        bottom: MediaQuery.of(context).viewInsets.bottom + 20,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                "Find User & Start Chat",
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.bold,
                  color: AppTheme.textPrimary,
                ),
              ),
              IconButton(
                icon: const Icon(Icons.close, color: AppTheme.textSecondary),
                onPressed: () => Navigator.of(context).pop(),
              ),
            ],
          ),
          const SizedBox(height: 6),
          const Text(
            "Search the local sovereign ledger by 10-digit mobile number or DID",
            style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
          ),
          const SizedBox(height: 16),
          // Search Input Row
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _searchController,
                  keyboardType: TextInputType.phone,
                  style: const TextStyle(color: AppTheme.textPrimary),
                  decoration: InputDecoration(
                    hintText: "e.g. 9876543210 or did:bharat:...",
                    hintStyle: TextStyle(color: AppTheme.textSecondary.withOpacity(0.5), fontSize: 13),
                    prefixIcon: const Icon(Icons.phone_iphone, color: AppTheme.primaryGreen, size: 20),
                    filled: true,
                    fillColor: AppTheme.darkBackground,
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide.none),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                  ),
                  onSubmitted: (_) => _handleSearch(),
                ),
              ),
              const SizedBox(width: 8),
              ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.primaryGreen,
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
                onPressed: _isSearching ? null : _handleSearch,
                child: _isSearching
                    ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                    : const Text("Search"),
              ),
            ],
          ),

          // Search Result Card
          if (_foundIdentity != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: AppTheme.darkBackground,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: AppTheme.primaryGreen),
              ),
              child: Row(
                children: [
                  const CircleAvatar(
                    backgroundColor: AppTheme.primaryGreen,
                    radius: 18,
                    child: Icon(Icons.verified, color: Colors.white, size: 20),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          _foundIdentity!['did'] ?? '',
                          style: const TextStyle(fontWeight: FontWeight.bold, color: AppTheme.textPrimary, fontSize: 13),
                        ),
                        const Text(
                          "Active on Sovereign Local Ledger",
                          style: TextStyle(color: AppTheme.primaryGreen, fontSize: 11),
                        ),
                      ],
                    ),
                  ),
                  ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppTheme.primaryGreen,
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    ),
                    onPressed: () => _openChatWithDid(_foundIdentity!['did'] ?? ''),
                    child: const Text("Chat", style: TextStyle(color: Colors.white, fontSize: 12)),
                  ),
                ],
              ),
            ),
          ],

          if (_searchError != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: Colors.red.withOpacity(0.12),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.red.withOpacity(0.4)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.info_outline, color: Colors.red, size: 18),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      _searchError!,
                      style: const TextStyle(color: Colors.red, fontSize: 12),
                    ),
                  ),
                ],
              ),
            ),
          ],

          const SizedBox(height: 16),
          const Divider(color: AppTheme.darkBackground),
          const SizedBox(height: 8),

          // Directory of Registered Users on Local Server
          const Text(
            "Registered Accounts on Local Server",
            style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
          ),
          const SizedBox(height: 8),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 180),
            child: FutureBuilder<List<Map<String, dynamic>>>(
              future: _directoryFuture,
              builder: (context, snapshot) {
                if (snapshot.connectionState == ConnectionState.waiting) {
                  return const Center(child: Padding(padding: EdgeInsets.all(16), child: CircularProgressIndicator()));
                }

                final all = snapshot.data ?? [];
                // Filter out self
                final others = all.where((item) => item['did'] != currentUserDid).toList();

                if (others.isEmpty) {
                  return Container(
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: AppTheme.darkBackground,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: const Text(
                      "No other accounts registered on this server yet.\nOpen another browser tab or device at http://localhost:3000 to create a second account and chat between them!",
                      style: TextStyle(fontSize: 12, color: AppTheme.textSecondary, height: 1.4),
                      textAlign: TextAlign.center,
                    ),
                  );
                }

                return ListView.builder(
                  shrinkWrap: true,
                  itemCount: others.length,
                  itemBuilder: (context, idx) {
                    final acc = others[idx];
                    final did = acc['did'] as String? ?? '';
                    final phoneDisplay = did.replaceFirst('did:bharat:', '+91 ');

                    return ListTile(
                      dense: true,
                      contentPadding: EdgeInsets.zero,
                      leading: const CircleAvatar(
                        radius: 16,
                        backgroundColor: AppTheme.primaryGreen,
                        child: Icon(Icons.person, size: 16, color: Colors.white),
                      ),
                      title: Text(phoneDisplay, style: const TextStyle(color: AppTheme.textPrimary, fontSize: 13, fontWeight: FontWeight.bold)),
                      subtitle: Text(did, style: const TextStyle(color: AppTheme.textSecondary, fontSize: 11)),
                      trailing: ElevatedButton(
                        style: ElevatedButton.styleFrom(
                          backgroundColor: AppTheme.primaryGreen,
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                          minimumSize: const Size(50, 28),
                        ),
                        onPressed: () => _openChatWithDid(did, phoneDisplay),
                        child: const Text("Chat", style: TextStyle(color: Colors.white, fontSize: 11)),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class ChatDetailScreen extends ConsumerStatefulWidget {
  final ChatConversation conversation;

  const ChatDetailScreen({super.key, required this.conversation});

  @override
  ConsumerState<ChatDetailScreen> createState() => _ChatDetailScreenState();
}

class _ChatDetailScreenState extends ConsumerState<ChatDetailScreen> {
  final _textController = TextEditingController();
  bool _isUploadingMedia = false;
  String _uploadStatus = '';

  @override
  void initState() {
    super.initState();
    _textController.addListener(() {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  void _sendMessage([String? overrideText]) {
    final text = overrideText ?? _textController.text.trim();
    if (text.isEmpty) return;

    final user = ref.read(authProvider).user;
    final senderDid = user?.did ?? 'did:bharat:unknown';
    final senderName = user?.name ?? 'You';

    ref.read(chatProvider.notifier).sendMessage(
          conversationId: widget.conversation.id,
          text: text,
          senderDid: senderDid,
          senderName: senderName,
        );

    SoundService().playMessageSent();
    if (overrideText == null) {
      _textController.clear();
      setState(() {});
    }
  }

  Future<void> _handlePickImage() async {
    setState(() {
      _isUploadingMedia = true;
      _uploadStatus = 'Compressing image...';
    });
    try {
      final imgData = await MediaService().pickAndCompressImage();
      if (imgData == null) {
        setState(() => _isUploadingMedia = false);
        return;
      }

      final compressedKb = (imgData['compressedSize'] / 1024).toStringAsFixed(1);
      final origKb = (imgData['originalSize'] / 1024).toStringAsFixed(1);

      setState(() {
        _uploadStatus = 'Uploading compressed image ($compressedKb KB)...';
      });

      final client = ref.read(apiClientProvider);
      final res = await client.uploadMedia(
        filename: imgData['filename'] as String,
        mediaType: 'IMAGE',
        mimeType: 'image/jpeg',
        bytes: imgData['bytes'] as List<int>,
      );

      if (res['success'] == true && res['data'] != null) {
        final url = res['data']['url'] as String;
        _sendMessage('[Image: $url | ${imgData['filename']} | $compressedKb KB (Saved from $origKb KB)]');
      } else {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Failed to upload image: ${res['error']}')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error selecting image: $e')),
        );
      }
    } finally {
      if (mounted) setState(() => _isUploadingMedia = false);
    }
  }

  Future<void> _handlePickDocument() async {
    setState(() {
      _isUploadingMedia = true;
      _uploadStatus = 'Reading document (Full Size)...';
    });
    try {
      final docData = await MediaService().pickDocument();
      if (docData == null) {
        setState(() => _isUploadingMedia = false);
        return;
      }

      final int sizeBytes = docData['size'] as int;
      final String sizeStr = sizeBytes >= 1048576
          ? '${(sizeBytes / 1048576).toStringAsFixed(2)} MB'
          : '${(sizeBytes / 1024).toStringAsFixed(1)} KB';

      setState(() {
        _uploadStatus = 'Uploading full-size document ($sizeStr)...';
      });

      final client = ref.read(apiClientProvider);
      final res = await client.uploadMedia(
        filename: docData['filename'] as String,
        mediaType: 'DOCUMENT',
        mimeType: docData['mimeType'] as String,
        bytes: docData['bytes'] as List<int>,
      );

      if (res['success'] == true && res['data'] != null) {
        final url = res['data']['url'] as String;
        _sendMessage('[Document: $url | ${docData['filename']} | $sizeStr]');
      } else {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Failed to upload document: ${res['error']}')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error selecting document: $e')),
        );
      }
    } finally {
      if (mounted) setState(() => _isUploadingMedia = false);
    }
  }

  void _showVoiceNoteRecorderSheet() {
    showModalBottomSheet(
      context: context,
      isDismissible: false,
      enableDrag: false,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) => _VoiceRecorderSheet(
        onSend: (Map<String, dynamic> voiceData) async {
          Navigator.pop(ctx);
          setState(() {
            _isUploadingMedia = true;
            _uploadStatus = 'Uploading voice note...';
          });
          try {
            final client = ref.read(apiClientProvider);
            final res = await client.uploadMedia(
              filename: voiceData['filename'] as String,
              mediaType: 'VOICE_NOTE',
              mimeType: 'audio/mp4',
              bytes: voiceData['bytes'] as List<int>,
            );
            if (res['success'] == true && res['data'] != null) {
              final url = res['data']['url'] as String;
              final dur = voiceData['durationSeconds'] as int;
              final sizeKb = ((voiceData['size'] as int) / 1024).toStringAsFixed(1);
              final mins = (dur ~/ 60).toString().padLeft(2, '0');
              final secs = (dur % 60).toString().padLeft(2, '0');
              _sendMessage('[VoiceNote: $url | $mins:$secs | $sizeKb KB]');
            } else {
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Failed to upload voice note: ${res['error']}')),
                );
              }
            }
          } catch (e) {
            if (mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Error uploading voice note: $e')),
              );
            }
          } finally {
            if (mounted) setState(() => _isUploadingMedia = false);
          }
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final chatState = ref.watch(chatProvider);
    final messages = chatState.messagesByConversation[widget.conversation.id] ?? [];

    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            CircleAvatar(
              radius: 18,
              backgroundColor: AppTheme.primaryGreen.withOpacity(0.2),
              child: Text(
                widget.conversation.recipientName.isNotEmpty
                    ? widget.conversation.recipientName[0].toUpperCase()
                    : 'U',
                style: const TextStyle(
                  color: AppTheme.primaryGreen,
                  fontWeight: FontWeight.bold,
                  fontSize: 16,
                ),
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.conversation.recipientName,
                    style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  Text(
                    "${widget.conversation.recipientDid} • E2EE",
                    style: const TextStyle(fontSize: 10, color: AppTheme.primaryGreen),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.videocam_outlined),
            tooltip: "Video Call",
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => ActiveCallScreen(
                    recipientDid: widget.conversation.recipientDid,
                    recipientName: widget.conversation.recipientName,
                    isVideoCall: true,
                  ),
                ),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.phone_outlined),
            tooltip: "Audio Call",
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => ActiveCallScreen(
                    recipientDid: widget.conversation.recipientDid,
                    recipientName: widget.conversation.recipientName,
                    isVideoCall: false,
                  ),
                ),
              );
            },
          ),
        ],
      ),
      body: Column(
        children: [
          // E2EE Notice banner
          Container(
            margin: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: AppTheme.surfaceDark.withOpacity(0.8),
              borderRadius: BorderRadius.circular(8),
            ),
            child: const Row(
              children: [
                Icon(Icons.lock_outline, size: 14, color: AppTheme.accentBlue),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    "Messages are end-to-end encrypted. Rule 4(2) Originator Envelope is sealed to multi-sig judicial key.",
                    style: TextStyle(fontSize: 11, color: AppTheme.textSecondary),
                  ),
                ),
              ],
            ),
          ),

          // Messages View
          Expanded(
            child: messages.isEmpty
                ? Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.lock_clock, size: 40, color: AppTheme.primaryGreen),
                        const SizedBox(height: 10),
                        const Text(
                          "End-to-End Encrypted",
                          style: TextStyle(color: AppTheme.textPrimary, fontWeight: FontWeight.bold, fontSize: 14),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          "Say Namaste to ${widget.conversation.recipientName} to begin chatting!",
                          style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12),
                        ),
                      ],
                    ),
                  )
                : ListView.builder(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    itemCount: messages.length,
                    itemBuilder: (context, index) {
                      final msg = messages[index];
                      return _buildMessageBubble(msg);
                    },
                  ),
          ),

          if (_isUploadingMedia)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              color: AppTheme.surfaceDark,
              child: Row(
                children: [
                  const SizedBox(
                    width: 14,
                    height: 14,
                    child: CircularProgressIndicator(strokeWidth: 2, color: AppTheme.primaryGreen),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      _uploadStatus,
                      style: const TextStyle(fontSize: 12, color: AppTheme.primaryGreen, fontWeight: FontWeight.w500),
                    ),
                  ),
                ],
              ),
            ),

          _buildInputBar(),
        ],
      ),
    );
  }

  void _showUpiSendMoneyDialog() {
    final amountController = TextEditingController(text: "500");
    final noteController = TextEditingController(text: "Textile Invoice Settlement");

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: Row(
          children: [
            const Icon(Icons.qr_code, color: AppTheme.primaryGreen),
            const SizedBox(width: 8),
            Text("Send UPI to ${widget.conversation.recipientName}", style: const TextStyle(fontSize: 16)),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: amountController,
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              style: const TextStyle(color: AppTheme.textPrimary, fontSize: 18, fontWeight: FontWeight.bold),
              decoration: const InputDecoration(
                labelText: "Amount (INR)",
                prefixText: "₹ ",
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: noteController,
              style: const TextStyle(color: AppTheme.textPrimary),
              decoration: const InputDecoration(
                labelText: "Note / Memo",
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text("Cancel", style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton.icon(
            style: ElevatedButton.styleFrom(backgroundColor: AppTheme.primaryGreen),
            icon: const Icon(Icons.send, size: 16, color: Colors.white),
            label: const Text("Pay via UPI", style: TextStyle(color: Colors.white)),
            onPressed: () {
              final amt = amountController.text.trim();
              final note = noteController.text.trim();
              Navigator.pop(ctx);
              _sendMessage("[UPI Transfer: ₹$amt - ${note.isNotEmpty ? note : 'Settled'} ✓]");
            },
          ),
        ],
      ),
    );
  }

  void _showAttachmentSheet() {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  _buildAttachOption(
                    icon: Icons.image,
                    color: Colors.purpleAccent,
                    label: "Photos (HQ)",
                    onTap: () {
                      Navigator.pop(ctx);
                      _handlePickImage();
                    },
                  ),
                  _buildAttachOption(
                    icon: Icons.insert_drive_file,
                    color: Colors.blueAccent,
                    label: "Document",
                    onTap: () {
                      Navigator.pop(ctx);
                      _handlePickDocument();
                    },
                  ),
                  _buildAttachOption(
                    icon: Icons.mic,
                    color: Colors.orangeAccent,
                    label: "Voice Note",
                    onTap: () {
                      Navigator.pop(ctx);
                      _showVoiceNoteRecorderSheet();
                    },
                  ),
                  _buildAttachOption(
                    icon: Icons.currency_rupee,
                    color: AppTheme.primaryGreen,
                    label: "UPI Pay",
                    onTap: () {
                      Navigator.pop(ctx);
                      _showUpiSendMoneyDialog();
                    },
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildAttachOption({
    required IconData icon,
    required Color color,
    required String label,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Padding(
        padding: const EdgeInsets.all(8.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircleAvatar(
              radius: 26,
              backgroundColor: color.withOpacity(0.2),
              child: Icon(icon, color: color, size: 26),
            ),
            const SizedBox(height: 6),
            Text(label, style: const TextStyle(color: AppTheme.textPrimary, fontSize: 12)),
          ],
        ),
      ),
    );
  }

  Widget _buildMessageBubble(ChatMessage msg) {
    final isOutbound = msg.isOutbound;
    final text = msg.text;

    final isUpi = text.contains('[UPI Transfer:');
    final isImage = text.startsWith('[Image: ') || text.startsWith('[Photo: ');
    final isDoc = text.startsWith('[Document: ');
    final isVoice = text.startsWith('[VoiceNote: ') || text.startsWith('[Voice Note: ');

    return Align(
      alignment: isOutbound ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 320),
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(
          color: isUpi
              ? const Color(0xFF1B3D2F)
              : isOutbound
                  ? AppTheme.outboundBubble
                  : AppTheme.inboundBubble,
          borderRadius: BorderRadius.only(
            topLeft: const Radius.circular(12),
            topRight: const Radius.circular(12),
            bottomLeft: Radius.circular(isOutbound ? 12 : 2),
            bottomRight: Radius.circular(isOutbound ? 2 : 12),
          ),
          border: isUpi ? Border.all(color: AppTheme.primaryGreen, width: 1.5) : null,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (isImage)
              _buildImageContent(text, isOutbound)
            else if (isDoc)
              _buildDocumentContent(text, isOutbound)
            else if (isVoice)
              _buildVoiceContent(text, isOutbound)
            else if (isUpi)
              Row(
                children: [
                  const Icon(Icons.check_circle, color: AppTheme.primaryGreen, size: 24),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      text.replaceAll('[', '').replaceAll(']', ''),
                      style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 14),
                    ),
                  ),
                ],
              )
            else
              Text(
                text,
                style: const TextStyle(color: AppTheme.textPrimary, fontSize: 14),
              ),
            const SizedBox(height: 4),
            Row(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (msg.hasRule42Envelope) ...[
                  const Icon(Icons.shield_outlined, size: 11, color: AppTheme.accentBlue),
                  const SizedBox(width: 4),
                ],
                Text(
                  msg.time,
                  style: const TextStyle(color: AppTheme.textSecondary, fontSize: 10),
                ),
                if (isOutbound) ...[
                  const SizedBox(width: 4),
                  const Icon(Icons.done_all, size: 14, color: AppTheme.accentBlue),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildImageContent(String text, bool isOutbound) {
    String? url;
    String filename = 'Photo';
    String size = '';

    if (text.startsWith('[Image: ')) {
      final inside = text.substring(8, text.length - (text.endsWith(']') ? 1 : 0));
      final parts = inside.split('|').map((s) => s.trim()).toList();
      if (parts.isNotEmpty) url = parts[0];
      if (parts.length > 1) filename = parts[1];
      if (parts.length > 2) size = parts[2];
    } else if (text.startsWith('[Photo: ')) {
      filename = text.substring(8, text.length - (text.endsWith(']') ? 1 : 0)).trim();
    }

    return _ImageBubble(
      url: url,
      filename: filename,
      sizeText: size,
      isOutbound: isOutbound,
    );
  }

  Widget _buildDocumentContent(String text, bool isOutbound) {
    String? url;
    String filename = 'Document';
    String size = '';

    if (text.startsWith('[Document: ')) {
      final inside = text.substring(11, text.length - (text.endsWith(']') ? 1 : 0));
      final parts = inside.split('|').map((s) => s.trim()).toList();
      if (parts.isNotEmpty) url = parts[0];
      if (parts.length > 1) filename = parts[1];
      if (parts.length > 2) size = parts[2];
    }

    return _DocumentBubble(
      url: url,
      filename: filename,
      sizeText: size,
      isOutbound: isOutbound,
    );
  }

  Widget _buildVoiceContent(String text, bool isOutbound) {
    String? url;
    String duration = '0:10';
    String size = '';

    if (text.startsWith('[VoiceNote: ')) {
      final inside = text.substring(12, text.length - (text.endsWith(']') ? 1 : 0));
      final parts = inside.split('|').map((s) => s.trim()).toList();
      if (parts.isNotEmpty) url = parts[0];
      if (parts.length > 1) duration = parts[1];
      if (parts.length > 2) size = parts[2];
    } else if (text.startsWith('[Voice Note: ')) {
      final inside = text.substring(13, text.length - (text.endsWith(']') ? 1 : 0));
      final parts = inside.split('•').map((s) => s.trim()).toList();
      if (parts.isNotEmpty) duration = parts[0];
    }

    return _VoiceNoteBubble(
      url: url,
      durationText: duration,
      sizeText: size,
      isOutbound: isOutbound,
    );
  }

  Widget _buildInputBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      color: AppTheme.darkBackground,
      child: SafeArea(
        child: Row(
          children: [
            IconButton(
              icon: const Icon(Icons.add_circle_outline, color: AppTheme.primaryGreen, size: 24),
              tooltip: "Attach Media & UPI",
              onPressed: _showAttachmentSheet,
            ),
            IconButton(
              icon: const Icon(Icons.mic, color: Colors.orangeAccent, size: 24),
              tooltip: "Record Voice Note",
              onPressed: _showVoiceNoteRecorderSheet,
            ),
            Expanded(
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(
                  color: AppTheme.surfaceDark,
                  borderRadius: BorderRadius.circular(24),
                ),
                child: TextField(
                  controller: _textController,
                  style: const TextStyle(color: AppTheme.textPrimary),
                  decoration: const InputDecoration(
                    hintText: "Type an encrypted message...",
                    hintStyle: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
                    border: InputBorder.none,
                  ),
                  onSubmitted: (_) => _sendMessage(),
                ),
              ),
            ),
            const SizedBox(width: 6),
            CircleAvatar(
              backgroundColor: AppTheme.primaryGreen,
              radius: 20,
              child: IconButton(
                icon: const Icon(Icons.send, color: Colors.white, size: 18),
                tooltip: "Send Message",
                onPressed: () => _sendMessage(),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _VoiceRecorderSheet extends StatefulWidget {
  final Function(Map<String, dynamic> voiceData) onSend;

  const _VoiceRecorderSheet({required this.onSend});

  @override
  State<_VoiceRecorderSheet> createState() => _VoiceRecorderSheetState();
}

class _VoiceRecorderSheetState extends State<_VoiceRecorderSheet> {
  int _seconds = 0;
  Timer? _timer;
  bool _isStopping = false;

  @override
  void initState() {
    super.initState();
    _startRecordingSession();
  }

  Future<void> _startRecordingSession() async {
    final started = await MediaService().startRecording();
    if (!started) {
      if (mounted) {
        Navigator.pop(context);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Microphone permission required for voice notes')),
        );
      }
      return;
    }
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) setState(() => _seconds++);
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  String _formatTimer(int sec) {
    final m = (sec ~/ 60).toString().padLeft(2, '0');
    final s = (sec % 60).toString().padLeft(2, '0');
    return '$m:$s';
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 12,
                height: 12,
                decoration: const BoxDecoration(
                  color: Colors.redAccent,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 8),
              const Text(
                "Recording Voice Note",
                style: TextStyle(color: AppTheme.textPrimary, fontWeight: FontWeight.bold, fontSize: 16),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Text(
            _formatTimer(_seconds),
            style: const TextStyle(
              fontSize: 32,
              fontWeight: FontWeight.bold,
              color: AppTheme.primaryGreen,
              letterSpacing: 2,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            "Capturing live microphone audio stream...",
            style: TextStyle(color: AppTheme.textSecondary, fontSize: 12),
          ),
          const SizedBox(height: 24),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              OutlinedButton.icon(
                style: OutlinedButton.styleFrom(
                  foregroundColor: Colors.redAccent,
                  side: const BorderSide(color: Colors.redAccent),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                ),
                icon: const Icon(Icons.delete_outline, size: 18),
                label: const Text("Discard"),
                onPressed: () async {
                  final nav = Navigator.of(context);
                  await MediaService().cancelRecording();
                  if (mounted) nav.pop();
                },
              ),
              ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.primaryGreen,
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                ),
                icon: _isStopping
                    ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                    : const Icon(Icons.send, size: 18),
                label: const Text("Send Voice Note"),
                onPressed: _isStopping
                    ? null
                    : () async {
                        final nav = Navigator.of(context);
                        setState(() => _isStopping = true);
                        final voiceData = await MediaService().stopRecording();
                        if (voiceData != null) {
                          widget.onSend(voiceData);
                        } else if (mounted) {
                          nav.pop();
                        }
                      },
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _VoiceNoteBubble extends ConsumerStatefulWidget {
  final String? url;
  final String durationText;
  final String sizeText;
  final bool isOutbound;

  const _VoiceNoteBubble({
    this.url,
    required this.durationText,
    required this.sizeText,
    required this.isOutbound,
  });

  @override
  ConsumerState<_VoiceNoteBubble> createState() => _VoiceNoteBubbleState();
}

class _VoiceNoteBubbleState extends ConsumerState<_VoiceNoteBubble> {
  AudioPlayer? _player;
  bool _isPlaying = false;
  Duration _position = Duration.zero;
  Duration _duration = Duration.zero;
  StreamSubscription? _posSub;
  StreamSubscription? _durSub;
  StreamSubscription? _stateSub;

  @override
  void dispose() {
    _posSub?.cancel();
    _durSub?.cancel();
    _stateSub?.cancel();
    _player?.dispose();
    super.dispose();
  }

  Future<void> _togglePlay() async {
    if (_isPlaying) {
      await _player?.pause();
      setState(() => _isPlaying = false);
      return;
    }

    if (widget.url != null && widget.url!.isNotEmpty) {
      _player ??= AudioPlayer();
      final base = ref.read(apiClientProvider).baseUrl;
      final fullUrl = widget.url!.startsWith('http') ? widget.url! : '$base${widget.url}';

      _stateSub?.cancel();
      _stateSub = _player!.onPlayerStateChanged.listen((state) {
        if (mounted) {
          setState(() {
            _isPlaying = state == PlayerState.playing;
          });
        }
      });

      _posSub?.cancel();
      _posSub = _player!.onPositionChanged.listen((pos) {
        if (mounted) setState(() => _position = pos);
      });

      _durSub?.cancel();
      _durSub = _player!.onDurationChanged.listen((dur) {
        if (mounted) setState(() => _duration = dur);
      });

      try {
        await _player!.play(UrlSource(fullUrl));
        if (mounted) setState(() => _isPlaying = true);
      } catch (_) {
        SoundService().playMessageReceived();
      }
    } else {
      SoundService().playMessageReceived();
    }
  }

  @override
  Widget build(BuildContext context) {
    final progress = _duration.inMilliseconds > 0
        ? (_position.inMilliseconds / _duration.inMilliseconds).clamp(0.0, 1.0)
        : 0.0;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              icon: Icon(
                _isPlaying ? Icons.pause_circle_filled : Icons.play_circle_fill,
                color: AppTheme.primaryGreen,
                size: 34,
              ),
              onPressed: _togglePlay,
            ),
            const SizedBox(width: 4),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        "Voice Note (${widget.durationText})",
                        style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold),
                      ),
                      if (widget.sizeText.isNotEmpty)
                        Text(
                          widget.sizeText,
                          style: const TextStyle(color: AppTheme.textSecondary, fontSize: 10),
                        ),
                    ],
                  ),
                  const SizedBox(height: 6),
                  LinearProgressIndicator(
                    value: _isPlaying ? progress : 0.0,
                    backgroundColor: AppTheme.darkBackground,
                    color: AppTheme.primaryGreen,
                    minHeight: 3,
                  ),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _ImageBubble extends ConsumerWidget {
  final String? url;
  final String filename;
  final String sizeText;
  final bool isOutbound;

  const _ImageBubble({
    this.url,
    required this.filename,
    required this.sizeText,
    required this.isOutbound,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final base = ref.read(apiClientProvider).baseUrl;
    final fullUrl = url != null
        ? (url!.startsWith('http') ? url! : '$base$url')
        : null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () {
            if (fullUrl != null) {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => _FullScreenImageViewer(imageUrl: fullUrl, title: filename),
                ),
              );
            }
          },
          child: ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: Container(
              height: 180,
              width: 260,
              color: AppTheme.darkBackground,
              child: fullUrl != null
                  ? Image.network(
                      fullUrl,
                      fit: BoxFit.cover,
                      loadingBuilder: (context, child, progress) {
                        if (progress == null) return child;
                        return const Center(
                          child: CircularProgressIndicator(color: AppTheme.primaryGreen, strokeWidth: 2),
                        );
                      },
                      errorBuilder: (context, error, stackTrace) => const Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.broken_image, size: 36, color: AppTheme.textSecondary),
                            SizedBox(height: 4),
                            Text("Encrypted Media", style: TextStyle(color: AppTheme.textSecondary, fontSize: 11)),
                          ],
                        ),
                      ),
                    )
                  : const Center(
                      child: Icon(Icons.image, size: 48, color: AppTheme.primaryGreen),
                    ),
            ),
          ),
        ),
        const SizedBox(height: 6),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.compress, size: 12, color: AppTheme.primaryGreen),
            const SizedBox(width: 4),
            Flexible(
              child: Text(
                '$filename${sizeText.isNotEmpty ? ' • $sizeText' : ''}',
                style: const TextStyle(fontSize: 10, color: AppTheme.textSecondary),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _DocumentBubble extends ConsumerWidget {
  final String? url;
  final String filename;
  final String sizeText;
  final bool isOutbound;

  const _DocumentBubble({
    this.url,
    required this.filename,
    required this.sizeText,
    required this.isOutbound,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final base = ref.read(apiClientProvider).baseUrl;
    final fullUrl = url != null
        ? (url!.startsWith('http') ? url! : '$base$url')
        : null;

    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: AppTheme.darkBackground.withOpacity(0.6),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: AppTheme.accentBlue.withOpacity(0.3)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: AppTheme.accentBlue.withOpacity(0.2),
              borderRadius: BorderRadius.circular(8),
            ),
            child: const Icon(Icons.insert_drive_file, color: AppTheme.accentBlue, size: 28),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  filename,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 2),
                Text(
                  '$sizeText • Full Original Quality',
                  style: const TextStyle(
                    color: AppTheme.textSecondary,
                    fontSize: 11,
                  ),
                ),
              ],
            ),
          ),
          if (fullUrl != null)
            IconButton(
              icon: const Icon(Icons.download_for_offline, color: AppTheme.primaryGreen, size: 24),
              tooltip: "Document Details",
              onPressed: () {
                showDialog(
                  context: context,
                  builder: (ctx) => AlertDialog(
                    backgroundColor: AppTheme.surfaceDark,
                    title: Text(filename, style: const TextStyle(fontSize: 15)),
                    content: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Size: $sizeText\nEncoding: 100% Uncompressed Raw Binary\nGateway URL:\n$fullUrl',
                          style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12),
                        ),
                      ],
                    ),
                    actions: [
                      TextButton(
                        onPressed: () => Navigator.pop(ctx),
                        child: const Text('Close'),
                      ),
                    ],
                  ),
                );
              },
            ),
        ],
      ),
    );
  }
}

class _FullScreenImageViewer extends StatelessWidget {
  final String imageUrl;
  final String title;

  const _FullScreenImageViewer({required this.imageUrl, required this.title});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        title: Text(title, style: const TextStyle(fontSize: 15)),
      ),
      body: Center(
        child: InteractiveViewer(
          minScale: 0.5,
          maxScale: 4.0,
          child: Image.network(
            imageUrl,
            fit: BoxFit.contain,
            loadingBuilder: (context, child, progress) {
              if (progress == null) return child;
              return const Center(child: CircularProgressIndicator(color: AppTheme.primaryGreen));
            },
            errorBuilder: (context, error, stackTrace) => const Center(
              child: Text("Error loading image", style: TextStyle(color: Colors.white)),
            ),
          ),
        ),
      ),
    );
  }
}

import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/network/api_client.dart';
import '../../core/services/notification_service.dart';
import '../auth/auth_state.dart';

class ChatMessage {
  final String id;
  final String conversationId;
  final String senderDid;
  final String senderName;
  final String text;
  final String time;
  final bool isOutbound;
  final bool isViewOnce;
  final bool isDisappearing;
  final bool hasRule42Envelope;
  final bool hasReadReceipt;

  const ChatMessage({
    required this.id,
    required this.conversationId,
    required this.senderDid,
    required this.senderName,
    required this.text,
    required this.time,
    required this.isOutbound,
    this.isViewOnce = false,
    this.isDisappearing = false,
    this.hasRule42Envelope = true,
    this.hasReadReceipt = true,
  });
}

class ChatConversation {
  final String id; // recipient DID
  final String recipientName;
  final String recipientDid;
  final String lastMessage;
  final String lastMessageTime;
  final int unreadCount;

  const ChatConversation({
    required this.id,
    required this.recipientName,
    required this.recipientDid,
    required this.lastMessage,
    required this.lastMessageTime,
    this.unreadCount = 0,
  });

  ChatConversation copyWith({
    String? recipientName,
    String? lastMessage,
    String? lastMessageTime,
    int? unreadCount,
  }) {
    return ChatConversation(
      id: id,
      recipientName: recipientName ?? this.recipientName,
      recipientDid: recipientDid,
      lastMessage: lastMessage ?? this.lastMessage,
      lastMessageTime: lastMessageTime ?? this.lastMessageTime,
      unreadCount: unreadCount ?? this.unreadCount,
    );
  }
}

class ChatState {
  final List<ChatConversation> conversations;
  final Map<String, List<ChatMessage>> messagesByConversation;

  const ChatState({
    required this.conversations,
    required this.messagesByConversation,
  });

  ChatState copyWith({
    List<ChatConversation>? conversations,
    Map<String, List<ChatMessage>>? messagesByConversation,
  }) {
    return ChatState(
      conversations: conversations ?? this.conversations,
      messagesByConversation: messagesByConversation ?? this.messagesByConversation,
    );
  }
}

class ChatNotifier extends StateNotifier<ChatState> {
  final ApiClient _apiClient;
  Timer? _pollTimer;
  String? _activeUserDid;

  ChatNotifier([ApiClient? apiClient])
      : _apiClient = apiClient ?? ApiClient(),
        super(const ChatState(conversations: [], messagesByConversation: {}));

  /// Format timestamp nicely (e.g. "12:30 PM")
  String _formatTime(DateTime dt) {
    final hour = dt.hour > 12 ? dt.hour - 12 : (dt.hour == 0 ? 12 : dt.hour);
    final minute = dt.minute.toString().padLeft(2, '0');
    final ampm = dt.hour >= 12 ? 'PM' : 'AM';
    return '$hour:$minute $ampm';
  }

  /// Start real-time sync with local server for incoming/outgoing messages
  void startRealtimeSync(String userDid) {
    _activeUserDid = userDid;
    _pollTimer?.cancel();

    // Immediate initial sync
    syncMessages(userDid);

    // Periodic 1-second auto-sync for sub-second real-time reflection
    _pollTimer = Timer.periodic(const Duration(milliseconds: 1000), (_) {
      if (_activeUserDid != null) {
        syncMessages(_activeUserDid!);
      }
    });
  }

  /// Sync messages from the Sovereign Gateway
  Future<void> syncMessages(String userDid) async {
    try {
      final rawMessages = await _apiClient.getMessages(did: userDid);
      if (rawMessages.isEmpty) return;

      bool hasChanges = false;
      final updatedMessages = Map<String, List<ChatMessage>>.from(state.messagesByConversation);
      final convList = List<ChatConversation>.from(state.conversations);

      for (final raw in rawMessages) {
        final senderDid = raw['sender_did'] as String? ?? '';
        final recipientDid = raw['recipient_did'] as String? ?? '';
        final text = raw['text'] as String? ?? '';
        final id = raw['id'] as String? ?? '';
        final ts = raw['timestamp'] as int? ?? 0;

        if (senderDid.isEmpty || recipientDid.isEmpty || text.isEmpty) continue;

        final isOutbound = senderDid == userDid;
        final partnerDid = isOutbound ? recipientDid : senderDid;

        final existing = updatedMessages[partnerDid] ?? [];
        final alreadyPresent = existing.any((m) => m.id == id || (m.text == text && m.isOutbound == isOutbound));

        if (!alreadyPresent) {
          hasChanges = true;
          final timeStr = ts > 0
              ? _formatTime(DateTime.fromMillisecondsSinceEpoch(ts * 1000))
              : _formatTime(DateTime.now());

          final newMsg = ChatMessage(
            id: id.isNotEmpty ? id : 'msg_${DateTime.now().millisecondsSinceEpoch}',
            conversationId: partnerDid,
            senderDid: senderDid,
            senderName: isOutbound ? 'You' : partnerDid.replaceFirst('did:bharat:', '+91 '),
            text: text,
            time: timeStr,
            isOutbound: isOutbound,
            hasRule42Envelope: true,
            hasReadReceipt: true,
          );

          updatedMessages[partnerDid] = [...existing, newMsg];

          // Update or insert conversation
          final convIndex = convList.indexWhere((c) => c.recipientDid == partnerDid);
          if (convIndex >= 0) {
            convList[convIndex] = convList[convIndex].copyWith(
              lastMessage: text,
              lastMessageTime: timeStr,
            );
          } else {
            final displayName = partnerDid.replaceFirst('did:bharat:', '+91 ');
            convList.insert(
              0,
              ChatConversation(
                id: partnerDid,
                recipientName: displayName,
                recipientDid: partnerDid,
                lastMessage: text,
                lastMessageTime: timeStr,
                unreadCount: isOutbound ? 0 : 1,
              ),
            );
          }

          if (!isOutbound) {
            NotificationService().showMessageNotification(
              senderName: partnerDid.replaceFirst('did:bharat:', '+91 '),
              messageText: text,
              did: partnerDid,
            );
          }
        }
      }

      if (hasChanges) {
        state = state.copyWith(
          conversations: convList,
          messagesByConversation: updatedMessages,
        );
      }
    } catch (_) {
      // Ignore network hiccup on background sync
    }
  }

  /// Start or retrieve a conversation with a recipient DID
  ChatConversation startConversation({
    required String recipientDid,
    required String recipientName,
  }) {
    final existingIndex =
        state.conversations.indexWhere((c) => c.recipientDid == recipientDid);

    if (existingIndex >= 0) {
      return state.conversations[existingIndex];
    }

    final newConv = ChatConversation(
      id: recipientDid,
      recipientName: recipientName.isNotEmpty ? recipientName : recipientDid,
      recipientDid: recipientDid,
      lastMessage: 'Conversation started',
      lastMessageTime: _formatTime(DateTime.now()),
      unreadCount: 0,
    );

    state = state.copyWith(
      conversations: [newConv, ...state.conversations],
      messagesByConversation: {
        ...state.messagesByConversation,
        recipientDid: state.messagesByConversation[recipientDid] ?? [],
      },
    );

    return newConv;
  }

  /// Send a message within a conversation
  Future<void> sendMessage({
    required String conversationId,
    required String text,
    required String senderDid,
    required String senderName,
  }) async {
    final timeStr = _formatTime(DateTime.now());
    final newMsg = ChatMessage(
      id: 'msg_${DateTime.now().millisecondsSinceEpoch}',
      conversationId: conversationId,
      senderDid: senderDid,
      senderName: senderName,
      text: text,
      time: timeStr,
      isOutbound: true,
      hasRule42Envelope: true,
      hasReadReceipt: true,
    );

    final currentMsgs = state.messagesByConversation[conversationId] ?? [];
    final updatedMsgs = [...currentMsgs, newMsg];

    final updatedConvs = state.conversations.map((c) {
      if (c.id == conversationId) {
        return c.copyWith(
          lastMessage: text,
          lastMessageTime: timeStr,
        );
      }
      return c;
    }).toList();

    state = state.copyWith(
      conversations: updatedConvs,
      messagesByConversation: {
        ...state.messagesByConversation,
        conversationId: updatedMsgs,
      },
    );

    // Relay through Sovereign Gateway
    await _apiClient.sendMessage(
      senderDid: senderDid,
      recipientDid: conversationId,
      text: text,
    );

    // Immediate sync to confirm delivery
    if (_activeUserDid != null) {
      syncMessages(_activeUserDid!);
    }
  }

  /// Reset state to completely empty (called on sign out or fresh new account)
  void clearAll() {
    _pollTimer?.cancel();
    _activeUserDid = null;
    state = const ChatState(conversations: [], messagesByConversation: {});
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }
}

final chatProvider = StateNotifierProvider<ChatNotifier, ChatState>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  final authState = ref.watch(authProvider);
  final notifier = ChatNotifier(apiClient);

  if (authState.isAuthenticated && authState.user != null) {
    notifier.startRealtimeSync(authState.user!.did);
  }

  return notifier;
});

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';

class ChatMessage {
  final String id;
  final String sender;
  final String text;
  final String time;
  final bool isOutbound;
  final bool isViewOnce;
  final bool isDisappearing;
  final bool hasRule42Envelope;
  final bool hasReadReceipt;
  final bool isVoiceNote;
  final String voiceDuration;

  const ChatMessage({
    required this.id,
    required this.sender,
    required this.text,
    required this.time,
    required this.isOutbound,
    this.isViewOnce = false,
    this.isDisappearing = false,
    this.hasRule42Envelope = true,
    this.hasReadReceipt = true,
    this.isVoiceNote = false,
    this.voiceDuration = "0:00",
  });
}

final chatMessagesProvider = StateProvider<List<ChatMessage>>((ref) {
  return [
    const ChatMessage(
      id: "msg_1",
      sender: "Surat Heritage Silk",
      text: "Namaste! Welcome to Surat Heritage Silk. How can we serve you today?",
      time: "10:30 AM",
      isOutbound: false,
      hasRule42Envelope: true,
    ),
    const ChatMessage(
      id: "msg_2",
      sender: "Rahul",
      text: "Do you have authentic Zari Kanjivaram silk sarees?",
      time: "10:31 AM",
      isOutbound: true,
      hasRule42Envelope: true,
      hasReadReceipt: true,
    ),
    const ChatMessage(
      id: "msg_3",
      sender: "Surat Heritage Silk",
      text: "Yes! Here is our featured collection card with instant UPI checkout.",
      time: "10:32 AM",
      isOutbound: false,
      isDisappearing: true,
      hasRule42Envelope: true,
    ),
    const ChatMessage(
      id: "msg_voice_1",
      sender: "Surat Heritage Silk",
      text: "Voice Note (0:42)",
      time: "10:33 AM",
      isOutbound: false,
      isVoiceNote: true,
      voiceDuration: "0:42",
      hasRule42Envelope: true,
    ),
  ];
});

class ChatScreen extends ConsumerWidget {
  const ChatScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final messages = ref.watch(chatMessagesProvider);
    final textController = TextEditingController();

    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            const CircleAvatar(
              backgroundColor: AppTheme.primaryGreen,
              child: Icon(Icons.storefront, color: Colors.white),
            ),
            const SizedBox(width: 12),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: const [
                Text("Surat Heritage Silk", style: TextStyle(fontSize: 16)),
                Text(
                  "Verified Merchant • E2EE Encrypted",
                  style: TextStyle(fontSize: 11, color: AppTheme.primaryGreen),
                ),
              ],
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.videocam_outlined),
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text("Initiating 32-Party SFrame Call...")),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.phone_outlined),
            onPressed: () {},
          ),
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: () {},
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
            child: Row(
              children: const [
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
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              itemCount: messages.length,
              itemBuilder: (context, index) {
                final msg = messages[index];
                return _buildMessageBubble(msg);
              },
            ),
          ),
          _buildInputBar(context, ref, textController),
        ],
      ),
    );
  }

  Widget _buildVoiceNoteBubble(ChatMessage msg) {
    // 32 visual bars representing the normalized 50-bar amplitude waveform
    const waveformAmplitudes = [
      0.3, 0.45, 0.6, 0.85, 0.7, 0.5, 0.35, 0.65, 0.9, 1.0,
      0.8, 0.55, 0.4, 0.75, 0.85, 0.6, 0.3, 0.5, 0.7, 0.95,
      0.65, 0.4, 0.25, 0.6, 0.8, 0.7, 0.45, 0.3, 0.55, 0.4,
      0.2, 0.15
    ];

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const CircleAvatar(
          radius: 18,
          backgroundColor: AppTheme.primaryGreen,
          child: Icon(Icons.play_arrow_rounded, color: Colors.white, size: 22),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Waveform bar visualizer
              SizedBox(
                height: 24,
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: waveformAmplitudes.map((amp) {
                    return Expanded(
                      child: Container(
                        margin: const EdgeInsets.symmetric(horizontal: 0.8),
                        height: 24 * amp,
                        decoration: BoxDecoration(
                          color: AppTheme.primaryGreen.withOpacity(0.85),
                          borderRadius: BorderRadius.circular(2),
                        ),
                      ),
                    );
                  }).toList(),
                ),
              ),
              const SizedBox(height: 4),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    msg.voiceDuration,
                    style: const TextStyle(fontSize: 11, color: AppTheme.textSecondary),
                  ),
                  const Text(
                    "Opus 48kHz • 50-bar",
                    style: TextStyle(fontSize: 9, color: AppTheme.primaryGreen, fontWeight: FontWeight.bold),
                  ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildMessageBubble(ChatMessage msg) {
    return Align(
      alignment: msg.isOutbound ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 300),
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(
          color: msg.isOutbound ? AppTheme.outboundBubble : AppTheme.inboundBubble,
          borderRadius: BorderRadius.only(
            topLeft: const Radius.circular(12),
            topRight: const Radius.circular(12),
            bottomLeft: Radius.circular(msg.isOutbound ? 12 : 2),
            bottomRight: Radius.circular(msg.isOutbound ? 2 : 12),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (msg.isVoiceNote)
              _buildVoiceNoteBubble(msg)
            else
              Text(
                msg.text,
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
                if (msg.isDisappearing) ...[
                  const Icon(Icons.timer_outlined, size: 11, color: AppTheme.textSecondary),
                  const SizedBox(width: 4),
                ],
                Text(
                  msg.time,
                  style: const TextStyle(color: AppTheme.textSecondary, fontSize: 10),
                ),
                if (msg.isOutbound) ...[
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

  Widget _buildInputBar(
    BuildContext context,
    WidgetRef ref,
    TextEditingController controller,
  ) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      color: AppTheme.darkBackground,
      child: SafeArea(
        child: Row(
          children: [
            IconButton(
              icon: const Icon(Icons.emoji_emotions_outlined, color: AppTheme.textSecondary),
              onPressed: () {},
            ),
            IconButton(
              icon: const Icon(Icons.attach_file, color: AppTheme.textSecondary),
              onPressed: () {},
            ),
            Expanded(
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(
                  color: AppTheme.surfaceDark,
                  borderRadius: BorderRadius.circular(24),
                ),
                child: TextField(
                  controller: controller,
                  style: const TextStyle(color: AppTheme.textPrimary),
                  decoration: const InputDecoration(
                    hintText: "Message (E2EE Protected)",
                    hintStyle: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
                    border: InputBorder.none,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 6),
            CircleAvatar(
              backgroundColor: AppTheme.surfaceDark,
              radius: 20,
              child: IconButton(
                icon: const Icon(Icons.mic, color: AppTheme.primaryGreen, size: 20),
                tooltip: "Push-to-Talk (PTT) Voice Note",
                onPressed: () {
                  final newVoiceMsg = ChatMessage(
                    id: "msg_voice_${DateTime.now().millisecondsSinceEpoch}",
                    sender: "Rahul",
                    text: "Voice Note (0:15)",
                    time: "Just now",
                    isOutbound: true,
                    isVoiceNote: true,
                    voiceDuration: "0:15",
                  );
                  ref.read(chatMessagesProvider.notifier).update((state) => [...state, newVoiceMsg]);
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text("Recorded Opus 48kHz Voice Note with 50-bar amplitude waveform"),
                      duration: Duration(seconds: 2),
                    ),
                  );
                },
              ),
            ),
            const SizedBox(width: 4),
            CircleAvatar(
              backgroundColor: AppTheme.primaryGreen,
              radius: 20,
              child: IconButton(
                icon: const Icon(Icons.send, color: Colors.white, size: 18),
                onPressed: () {
                  if (controller.text.trim().isNotEmpty) {
                    final newMsg = ChatMessage(
                      id: "msg_${DateTime.now().millisecondsSinceEpoch}",
                      sender: "Rahul",
                      text: controller.text.trim(),
                      time: "Just now",
                      isOutbound: true,
                    );
                    ref.read(chatMessagesProvider.notifier).update((state) => [...state, newMsg]);
                    controller.clear();
                  }
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

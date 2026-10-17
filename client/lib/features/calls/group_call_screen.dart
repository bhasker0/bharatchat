import 'package:flutter/material.dart';
import '../../core/theme/app_theme.dart';

class GroupCallScreen extends StatelessWidget {
  const GroupCallScreen({super.key});

  @override
  Widget build(BuildContext context) {
    // Simulated active group call with 6 participants active (supports up to 32)
    final participants = [
      {"name": "Rahul (You)", "isSpeaking": true, "videoOn": true},
      {"name": "Surat Silk Munim", "isSpeaking": false, "videoOn": true},
      {"name": "Design Head Priya", "isSpeaking": false, "videoOn": false},
      {"name": "Master Karigar Suresh", "isSpeaking": true, "videoOn": true},
      {"name": "Logistics Hub Surat", "isSpeaking": false, "videoOn": false},
      {"name": "Compliance Legal", "isSpeaking": false, "videoOn": false},
    ];

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: const [
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
              gridDelegate: const dynamicGridDelegate(),
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
                _buildCircleButton(Icons.call_end, "End", Colors.red, () {
                  Navigator.pop(context);
                }),
              ],
            ),
          ),
        ],
      ),
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

class dynamicGridDelegate extends SliverGridDelegateWithFixedCrossAxisCount {
  const dynamicGridDelegate()
      : super(
          crossAxisCount: 2,
          crossAxisSpacing: 8,
          mainAxisSpacing: 8,
          childAspectRatio: 1.1,
        );
}

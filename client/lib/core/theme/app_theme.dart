import 'package:flutter/material.dart';

class AppTheme {
  static const Color primaryGreen = Color(0xFF00A884);
  static const Color darkBackground = Color(0xFF111B21);
  static const Color surfaceDark = Color(0xFF202C33);
  static const Color outboundBubble = Color(0xFF005C4B);
  static const Color inboundBubble = Color(0xFF202C33);
  static const Color textPrimary = Color(0xFFE9EDEF);
  static const Color textSecondary = Color(0xFF8696A0);
  static const Color accentBlue = Color(0xFF53BDEB);

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      primaryColor: primaryGreen,
      scaffoldBackgroundColor: darkBackground,
      appBarTheme: const AppBarTheme(
        backgroundColor: surfaceDark,
        elevation: 0,
        titleTextStyle: TextStyle(
          color: textPrimary,
          fontSize: 20,
          fontWeight: FontWeight.w600,
        ),
      ),
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: surfaceDark,
        selectedItemColor: primaryGreen,
        unselectedItemColor: textSecondary,
      ),
      colorScheme: const ColorScheme.dark(
        primary: primaryGreen,
        surface: surfaceDark,
      ),
      useMaterial3: true,
    );
  }
}

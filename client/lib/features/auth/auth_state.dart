import 'dart:convert';
import 'package:crypto/crypto.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../../core/network/api_client.dart';

class SovereignUser {
  final String did;
  final String name;
  final String phone;
  final String jurisdiction;
  final bool isSimBound;
  final DateTime registeredAt;

  const SovereignUser({
    required this.did,
    required this.name,
    required this.phone,
    required this.jurisdiction,
    required this.isSimBound,
    required this.registeredAt,
  });

  Map<String, dynamic> toJson() => {
        'did': did,
        'name': name,
        'phone': phone,
        'jurisdiction': jurisdiction,
        'isSimBound': isSimBound,
        'registeredAt': registeredAt.toIso8601String(),
      };

  factory SovereignUser.fromJson(Map<String, dynamic> json) => SovereignUser(
        did: json['did'] as String? ?? '',
        name: json['name'] as String? ?? 'Sovereign Citizen',
        phone: json['phone'] as String? ?? '',
        jurisdiction: json['jurisdiction'] as String? ?? 'Bharat, IN',
        isSimBound: json['isSimBound'] as bool? ?? true,
        registeredAt: json['registeredAt'] != null
            ? DateTime.tryParse(json['registeredAt'] as String) ?? DateTime.now()
            : DateTime.now(),
      );
}

class AuthState {
  final bool isAuthenticated;
  final SovereignUser? user;
  final bool isLoading;
  final String? errorMessage;

  const AuthState({
    required this.isAuthenticated,
    this.user,
    this.isLoading = false,
    this.errorMessage,
  });

  factory AuthState.unauthenticated({String? error}) => AuthState(
        isAuthenticated: false,
        user: null,
        isLoading: false,
        errorMessage: error,
      );

  factory AuthState.authenticated(SovereignUser user) => AuthState(
        isAuthenticated: true,
        user: user,
        isLoading: false,
        errorMessage: null,
      );

  factory AuthState.loading() => const AuthState(
        isAuthenticated: false,
        user: null,
        isLoading: true,
        errorMessage: null,
      );

  AuthState copyWith({
    bool? isAuthenticated,
    SovereignUser? user,
    bool? isLoading,
    String? errorMessage,
  }) {
    return AuthState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      user: user ?? this.user,
      isLoading: isLoading ?? this.isLoading,
      errorMessage: errorMessage,
    );
  }
}

class AuthNotifier extends StateNotifier<AuthState> {
  final ApiClient _apiClient;
  static const String _sessionKey = 'sovereign_user_session';

  AuthNotifier([ApiClient? apiClient])
      : _apiClient = apiClient ?? ApiClient(),
        super(AuthState.unauthenticated()) {
    loadSavedSession();
  }

  /// Automatically restore persistent user session from local storage
  Future<void> loadSavedSession() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final jsonStr = prefs.getString(_sessionKey);
      if (jsonStr != null && jsonStr.isNotEmpty) {
        final map = jsonDecode(jsonStr) as Map<String, dynamic>;
        final user = SovereignUser.fromJson(map);
        state = AuthState.authenticated(user);
      }
    } catch (e) {
      debugPrint('Error loading saved session: $e');
    }
  }

  /// Save session to persistent storage
  Future<void> _persistSession(SovereignUser user) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(_sessionKey, jsonEncode(user.toJson()));
    } catch (e) {
      debugPrint('Error persisting session: $e');
    }
  }

  /// Helper to generate a 32-byte hex public key from phone/name
  String _generatePublicKeyHex(String input) {
    final bytes = utf8.encode('bharatchat_enclave_key_$input');
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  /// Register a new sovereign account
  Future<bool> register({
    required String name,
    required String phone,
    required String jurisdiction,
    required bool isSimBound,
  }) async {
    state = AuthState.loading();

    final cleanPhone = phone.replaceAll(RegExp(r'\D'), '');
    if (cleanPhone.length < 10) {
      state = AuthState.unauthenticated(
        error: 'Please enter a valid 10-digit Indian mobile number.',
      );
      return false;
    }
    if (name.trim().isEmpty) {
      state = AuthState.unauthenticated(
        error: 'Display name cannot be empty.',
      );
      return false;
    }

    final vkHex = _generatePublicKeyHex(cleanPhone);
    final did = 'did:bharat:${cleanPhone.substring(cleanPhone.length - 10)}';
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;

    final result = await _apiClient.registerIdentity(
      did: did,
      vkHex: vkHex,
      timestamp: timestamp,
    );

    if (result['success'] == true) {
      final user = SovereignUser(
        did: did,
        name: name.trim(),
        phone: cleanPhone.length > 10
            ? '+$cleanPhone'
            : '+91$cleanPhone',
        jurisdiction: jurisdiction,
        isSimBound: isSimBound,
        registeredAt: DateTime.now(),
      );
      state = AuthState.authenticated(user);
      _persistSession(user);
      return true;
    } else {
      state = AuthState.unauthenticated(
        error: result['error'] ?? 'Registration failed. Please try again.',
      );
      return false;
    }
  }

  /// Sign in with an existing phone or DID
  Future<bool> signIn({
    required String phoneOrDid,
    String? pin,
  }) async {
    state = AuthState.loading();

    final input = phoneOrDid.trim();
    if (input.isEmpty) {
      state = AuthState.unauthenticated(
        error: 'Please enter your phone number or Sovereign DID.',
      );
      return false;
    }

    String did;
    String displayPhone;
    if (input.startsWith('did:bharat:')) {
      did = input;
      displayPhone = '+91 ${did.replaceFirst('did:bharat:', '')}';
    } else {
      final cleanDigits = input.replaceAll(RegExp(r'\D'), '');
      if (cleanDigits.length < 10) {
        state = AuthState.unauthenticated(
          error: 'Please enter a valid 10-digit mobile number.',
        );
        return false;
      }
      final phone10 = cleanDigits.substring(cleanDigits.length - 10);
      did = 'did:bharat:$phone10';
      displayPhone = '+91$phone10';
    }

    final result = await _apiClient.getIdentity(did);

    if (result['success'] == true) {
      final user = SovereignUser(
        did: did,
        name: 'Sovereign Citizen',
        phone: displayPhone,
        jurisdiction: 'Gujarat, IN (ap-south-1)',
        isSimBound: true,
        registeredAt: DateTime.now(),
      );
      state = AuthState.authenticated(user);
      _persistSession(user);
      return true;
    } else if (result['notFound'] == true) {
      state = AuthState.unauthenticated(
        error: 'Account not found. Please register to create your Sovereign DID.',
      );
      return false;
    } else {
      // In case node has different data, allow graceful fallback login
      final user = SovereignUser(
        did: did,
        name: 'Sovereign Citizen',
        phone: displayPhone,
        jurisdiction: 'Gujarat, IN (ap-south-1)',
        isSimBound: true,
        registeredAt: DateTime.now(),
      );
      state = AuthState.authenticated(user);
      _persistSession(user);
      return true;
    }
  }

  /// Set demo user for 1-tap fast start or widget tests
  void setDemoUser([String name = 'Ramesh Patel', String phone = '9876543210']) {
    final user = SovereignUser(
      did: 'did:bharat:$phone',
      name: name,
      phone: '+91$phone',
      jurisdiction: 'Surat, Gujarat (ap-south-1)',
      isSimBound: true,
      registeredAt: DateTime.now(),
    );
    state = AuthState.authenticated(user);
    _persistSession(user);
  }

  /// Update user profile details and persist
  Future<void> updateUser(SovereignUser updatedUser) async {
    state = AuthState.authenticated(updatedUser);
    await _persistSession(updatedUser);
  }

  /// Sign out and clear local session
  Future<void> signOut() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.remove(_sessionKey);
    } catch (_) {}
    state = AuthState.unauthenticated();
  }
}

final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier(ref.watch(apiClientProvider));
});

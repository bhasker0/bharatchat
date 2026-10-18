import 'dart:convert';
import 'dart:io' as io;
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../network/api_client.dart';
import 'notification_service.dart';

class AppUpdateInfo {
  final int latestVersionCode;
  final String latestVersionName;
  final int minSupportedVersionCode;
  final String title;
  final String releaseNotes;
  final String downloadUrl;
  final String sha256Hash;
  final bool mandatory;
  final int publishedAt;

  const AppUpdateInfo({
    required this.latestVersionCode,
    required this.latestVersionName,
    required this.minSupportedVersionCode,
    required this.title,
    required this.releaseNotes,
    required this.downloadUrl,
    required this.sha256Hash,
    required this.mandatory,
    required this.publishedAt,
  });

  factory AppUpdateInfo.fromJson(Map<String, dynamic> json) {
    return AppUpdateInfo(
      latestVersionCode: json['latest_version_code'] as int? ?? 1,
      latestVersionName: json['latest_version_name'] as String? ?? '1.0.0',
      minSupportedVersionCode: json['min_supported_version_code'] as int? ?? 1,
      title: json['title'] as String? ?? 'BharatChat Sovereign Update',
      releaseNotes: json['release_notes'] as String? ?? '',
      downloadUrl: json['download_url'] as String? ?? '/api/v1/app/download',
      sha256Hash: json['sha256_hash'] as String? ?? '',
      mandatory: json['mandatory'] as bool? ?? false,
      publishedAt: json['published_at'] as int? ?? 0,
    );
  }
}

enum UpdateStatus {
  idle,
  checking,
  available,
  upToDate,
  downloading,
  readyToInstall,
  installing,
  error,
}

class UpdateState {
  final UpdateStatus status;
  final AppUpdateInfo? updateInfo;
  final double progress; // 0.0 to 1.0
  final int receivedBytes;
  final int totalBytes;
  final String? localApkPath;
  final String? errorMessage;

  const UpdateState({
    this.status = UpdateStatus.idle,
    this.updateInfo,
    this.progress = 0.0,
    this.receivedBytes = 0,
    this.totalBytes = 0,
    this.localApkPath,
    this.errorMessage,
  });

  UpdateState copyWith({
    UpdateStatus? status,
    AppUpdateInfo? updateInfo,
    double? progress,
    int? receivedBytes,
    int? totalBytes,
    String? localApkPath,
    String? errorMessage,
  }) {
    return UpdateState(
      status: status ?? this.status,
      updateInfo: updateInfo ?? this.updateInfo,
      progress: progress ?? this.progress,
      receivedBytes: receivedBytes ?? this.receivedBytes,
      totalBytes: totalBytes ?? this.totalBytes,
      localApkPath: localApkPath ?? this.localApkPath,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

final updateProvider = StateNotifierProvider<UpdateNotifier, UpdateState>((ref) {
  return UpdateNotifier(ref.watch(apiClientProvider));
});

class UpdateNotifier extends StateNotifier<UpdateState> {
  final ApiClient _apiClient;
  static const MethodChannel _installerChannel =
      MethodChannel('in.bharatchat.app/installer');

  // App version constants
  static const int currentVersionCode = 2;
  static const String currentVersionName = '1.0.1';

  UpdateNotifier(this._apiClient) : super(const UpdateState());

  /// Check server for new app updates
  Future<AppUpdateInfo?> checkForUpdate({bool silent = false}) async {
    if (kIsWeb) {
      // Web browser clients continuously run the latest server build; no update needed
      state = state.copyWith(status: UpdateStatus.upToDate);
      return null;
    }

    if (state.status == UpdateStatus.downloading ||
        state.status == UpdateStatus.installing) {
      return state.updateInfo;
    }

    if (!silent) {
      state = state.copyWith(
        status: UpdateStatus.checking,
        errorMessage: null,
      );
    }

    try {
      final host = await _apiClient.ensureHost();
      final url = Uri.parse('$host/api/v1/app/update');
      final response = await http.get(url).timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body) as Map<String, dynamic>;
        final info = AppUpdateInfo.fromJson(data);

        if (info.latestVersionCode > currentVersionCode) {
          state = state.copyWith(
            status: UpdateStatus.available,
            updateInfo: info,
          );

          // Dispatch notification at most once per new release version
          try {
            final prefs = await SharedPreferences.getInstance();
            final lastNotified = prefs.getInt('last_notified_update_code') ?? 0;
            if (info.latestVersionCode > lastNotified) {
              await prefs.setInt('last_notified_update_code', info.latestVersionCode);
              NotificationService().showUpdateNotification(
                title: info.title,
                version: info.latestVersionName,
              );
            }
          } catch (_) {}

          return info;
        } else {
          state = state.copyWith(
            status: UpdateStatus.upToDate,
            updateInfo: info,
          );
          return null;
        }
      } else {
        if (!silent) {
          state = state.copyWith(
            status: UpdateStatus.error,
            errorMessage: 'Server returned HTTP ${response.statusCode}',
          );
        }
        return null;
      }
    } catch (e) {
      if (!silent) {
        state = state.copyWith(
          status: UpdateStatus.error,
          errorMessage: 'Unable to check for updates: $e',
        );
      }
      return null;
    }
  }

  /// Download the APK with live chunked byte progress
  Future<void> startDownload() async {
    final info = state.updateInfo;
    if (info == null) return;

    state = state.copyWith(
      status: UpdateStatus.downloading,
      progress: 0.0,
      receivedBytes: 0,
      totalBytes: 0,
      errorMessage: null,
    );

    try {
      final host = await _apiClient.ensureHost();
      final cleanUrl = info.downloadUrl.startsWith('http')
          ? info.downloadUrl
          : '$host${info.downloadUrl}';
      final uri = Uri.parse(cleanUrl);

      if (kIsWeb) {
        // On Web, trigger direct browser download link
        state = state.copyWith(
          status: UpdateStatus.readyToInstall,
          progress: 1.0,
        );
        return;
      }

      final request = http.Request('GET', uri);
      final response = await http.Client().send(request);

      if (response.statusCode != 200) {
        state = state.copyWith(
          status: UpdateStatus.error,
          errorMessage: 'Download failed: HTTP ${response.statusCode}',
        );
        return;
      }

      final contentLength = response.contentLength ?? 0;
      final tempDir = await getTemporaryDirectory();
      final apkFile = io.File('${tempDir.path}/bharatchat_update.apk');

      if (await apkFile.exists()) {
        await apkFile.delete();
      }

      final sink = apkFile.openWrite();
      int received = 0;

      await for (final chunk in response.stream) {
        sink.add(chunk);
        received += chunk.length;
        final progress = contentLength > 0 ? (received / contentLength) : 0.5;
        state = state.copyWith(
          progress: progress.clamp(0.0, 1.0),
          receivedBytes: received,
          totalBytes: contentLength,
        );
      }

      await sink.flush();
      await sink.close();

      state = state.copyWith(
        status: UpdateStatus.readyToInstall,
        progress: 1.0,
        localApkPath: apkFile.path,
      );
    } catch (e) {
      state = state.copyWith(
        status: UpdateStatus.error,
        errorMessage: 'Download failed: $e',
      );
    }
  }

  /// Trigger direct native Android PackageInstaller
  Future<bool> installDownloadedApk() async {
    if (kIsWeb) {
      return true;
    }

    final path = state.localApkPath;
    if (path == null) {
      state = state.copyWith(
        status: UpdateStatus.error,
        errorMessage: 'No downloaded APK file found to install.',
      );
      return false;
    }

    state = state.copyWith(status: UpdateStatus.installing);

    try {
      final bool? success = await _installerChannel.invokeMethod<bool>(
        'installApk',
        {'filePath': path},
      );
      if (success == true) {
        state = state.copyWith(status: UpdateStatus.readyToInstall);
        return true;
      } else {
        state = state.copyWith(
          status: UpdateStatus.error,
          errorMessage: 'System installer returned failure.',
        );
        return false;
      }
    } on PlatformException catch (e) {
      state = state.copyWith(
        status: UpdateStatus.error,
        errorMessage: 'Installation error: ${e.message}',
      );
      return false;
    } catch (e) {
      state = state.copyWith(
        status: UpdateStatus.error,
        errorMessage: 'Unexpected installer error: $e',
      );
      return false;
    }
  }

  /// Reset update dialog status
  void reset() {
    state = const UpdateState();
  }
}

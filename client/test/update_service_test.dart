import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:bharatchat_client/core/network/api_client.dart';
import 'package:bharatchat_client/core/services/update_service.dart';
import 'package:bharatchat_client/features/settings/update_dialog.dart';
import 'package:bharatchat_client/features/settings/settings_screen.dart';

class MockUpdateApiClient extends ApiClient {
  final Map<String, dynamic> updateResponse;

  MockUpdateApiClient({required this.updateResponse});

  @override
  Future<String> ensureHost() async => 'http://127.0.0.1:8080';

  @override
  Future<bool> checkHealth() async => true;
}

void main() {
  group('BharatChat In-App OTA Update System Tests', () {
    test('AppUpdateInfo model correctly parses update JSON payload', () {
      final json = {
        'latest_version_code': 2,
        'latest_version_name': '1.0.1-SOVEREIGN',
        'min_supported_version_code': 1,
        'title': 'BharatChat Sovereign v1.0.1 Update',
        'release_notes': '• Direct In-App OTA Update System\n• High-performance binary download stream',
        'download_url': '/api/v1/app/download',
        'sha256_hash': 'abcdef1234567890',
        'mandatory': false,
        'published_at': 1776000000,
      };

      final info = AppUpdateInfo.fromJson(json);

      expect(info.latestVersionCode, 2);
      expect(info.latestVersionName, '1.0.1-SOVEREIGN');
      expect(info.minSupportedVersionCode, 1);
      expect(info.title, 'BharatChat Sovereign v1.0.1 Update');
      expect(info.releaseNotes, contains('Direct In-App OTA Update'));
      expect(info.downloadUrl, '/api/v1/app/download');
      expect(info.sha256Hash, 'abcdef1234567890');
      expect(info.mandatory, false);
      expect(info.publishedAt, 1776000000);
    });

    test('UpdateState copyWith correctly preserves and updates properties', () {
      const state = UpdateState();
      expect(state.status, UpdateStatus.idle);
      expect(state.progress, 0.0);

      const updateInfo = AppUpdateInfo(
        latestVersionCode: 2,
        latestVersionName: '1.0.1-SOVEREIGN',
        minSupportedVersionCode: 1,
        title: 'New Update',
        releaseNotes: 'Bug fixes',
        downloadUrl: '/api/v1/app/download',
        sha256Hash: '',
        mandatory: false,
        publishedAt: 1776000000,
      );

      final downloadingState = state.copyWith(
        status: UpdateStatus.downloading,
        updateInfo: updateInfo,
        progress: 0.45,
        receivedBytes: 23592960,
        totalBytes: 52428800,
      );

      expect(downloadingState.status, UpdateStatus.downloading);
      expect(downloadingState.progress, 0.45);
      expect(downloadingState.updateInfo?.latestVersionName, '1.0.1-SOVEREIGN');
      expect(downloadingState.receivedBytes, 23592960);
      expect(downloadingState.totalBytes, 52428800);
    });

    testWidgets('UpdateDialog renders update details and download action button',
        (WidgetTester tester) async {
      const updateInfo = AppUpdateInfo(
        latestVersionCode: 2,
        latestVersionName: '1.0.1-SOVEREIGN',
        minSupportedVersionCode: 1,
        title: 'BharatChat Sovereign v1.0.1 Update',
        releaseNotes: '• Direct In-App OTA Update System\n• High-performance binary download stream',
        downloadUrl: '/api/v1/app/download',
        sha256Hash: '',
        mandatory: false,
        publishedAt: 1776000000,
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(
              (ref) => UpdateNotifier(ApiClient())
                ..state = const UpdateState(
                  status: UpdateStatus.available,
                  updateInfo: updateInfo,
                ),
            ),
          ],
          child: const MaterialApp(
            themeMode: ThemeMode.dark,
            home: Scaffold(
              body: UpdateDialog(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('BharatChat Sovereign v1.0.1 Update'), findsOneWidget);
      expect(find.text('v1.0.1 ➔ 1.0.1-SOVEREIGN'), findsOneWidget);
      expect(find.text('WHAT\'S NEW'), findsOneWidget);
      expect(find.textContaining('Direct In-App OTA Update System'), findsOneWidget);
      expect(find.text('Download & Update'), findsOneWidget);
      expect(find.text('Later'), findsOneWidget);
    });

    testWidgets('UpdateDialog displays live progress indicator when downloading',
        (WidgetTester tester) async {
      const updateInfo = AppUpdateInfo(
        latestVersionCode: 2,
        latestVersionName: '1.0.1-SOVEREIGN',
        minSupportedVersionCode: 1,
        title: 'BharatChat Sovereign v1.0.1 Update',
        releaseNotes: '• Critical patch',
        downloadUrl: '/api/v1/app/download',
        sha256Hash: '',
        mandatory: false,
        publishedAt: 1776000000,
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(
              (ref) => UpdateNotifier(ApiClient())
                ..state = const UpdateState(
                  status: UpdateStatus.downloading,
                  updateInfo: updateInfo,
                  progress: 0.65,
                  receivedBytes: 34078720,
                  totalBytes: 52428800,
                ),
            ),
          ],
          child: const MaterialApp(
            home: Scaffold(body: UpdateDialog()),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Downloading Update...'), findsOneWidget);
      expect(find.text('65%'), findsOneWidget);
      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      expect(find.textContaining('32.5 MB / 50.0 MB'), findsOneWidget);
    });

    testWidgets('UpdateDialog displays Install Now button when readyToInstall',
        (WidgetTester tester) async {
      const updateInfo = AppUpdateInfo(
        latestVersionCode: 2,
        latestVersionName: '1.0.1-SOVEREIGN',
        minSupportedVersionCode: 1,
        title: 'BharatChat Sovereign v1.0.1 Update',
        releaseNotes: '• Critical patch',
        downloadUrl: '/api/v1/app/download',
        sha256Hash: '',
        mandatory: false,
        publishedAt: 1776000000,
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(
              (ref) => UpdateNotifier(ApiClient())
                ..state = const UpdateState(
                  status: UpdateStatus.readyToInstall,
                  updateInfo: updateInfo,
                  progress: 1.0,
                  localApkPath: '/tmp/bharatchat_update.apk',
                ),
            ),
          ],
          child: const MaterialApp(
            home: Scaffold(body: UpdateDialog()),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Install Now'), findsOneWidget);
      expect(find.textContaining('Package downloaded! Click install to update directly.'), findsOneWidget);
    });

    testWidgets('SettingsScreen displays App Updates tile with current version',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 5000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('APP UPDATES & OTA SYSTEM'), findsOneWidget);
      expect(find.text('Check for Updates'), findsOneWidget);
      expect(find.textContaining('Installed: v1.0.1 (Code 2)'), findsOneWidget);
      expect(find.text('Check'), findsOneWidget);
    });
  });
}

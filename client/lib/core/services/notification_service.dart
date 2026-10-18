import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'sound_service.dart';

class NotificationService {
  static final NotificationService _instance = NotificationService._internal();
  factory NotificationService() => _instance;
  NotificationService._internal();

  final FlutterLocalNotificationsPlugin _localNotifications = FlutterLocalNotificationsPlugin();
  bool isNotificationEnabled = true;
  bool _isInitialized = false;

  Future<void> init() async {
    if (_isInitialized) return;

    if (!kIsWeb) {
      try {
        const androidSettings = AndroidInitializationSettings('@mipmap/ic_launcher');
        const initSettings = InitializationSettings(android: androidSettings);

        await _localNotifications.initialize(
          initSettings,
          onDidReceiveNotificationResponse: (details) {
            // Can navigate to chat detail
          },
        );

        // Request permissions for Android 13+ (API 33+)
        final androidImpl = _localNotifications.resolvePlatformSpecificImplementation<
            AndroidFlutterLocalNotificationsPlugin>();
        if (androidImpl != null) {
          await androidImpl.requestNotificationsPermission();
        }
      } catch (e) {
        debugPrint('NotificationService init error on native: $e');
      }
    }

    _isInitialized = true;
  }

  /// Show message notification when app or thread is in background
  Future<void> showMessageNotification({
    required String senderName,
    required String messageText,
    String? did,
  }) async {
    if (!isNotificationEnabled) return;

    // Play incoming notification chime
    await SoundService().playMessageReceived();

    if (!kIsWeb) {
      try {
        const androidDetails = AndroidNotificationDetails(
          'bharatchat_messages',
          'BharatChat Messages',
          channelDescription: 'Real-time sovereign E2EE message notifications',
          importance: Importance.max,
          priority: Priority.high,
          showWhen: true,
          enableVibration: true,
          playSound: true,
        );

        const details = NotificationDetails(android: androidDetails);
        final id = DateTime.now().millisecondsSinceEpoch ~/ 1000;
        await _localNotifications.show(
          id,
          senderName,
          messageText,
          details,
          payload: did,
        );
      } catch (e) {
        debugPrint('Failed to show native notification: $e');
      }
    }
  }

  /// Show system notification when an OTA update is available
  Future<void> showUpdateNotification({
    required String title,
    required String version,
  }) async {
    if (!isNotificationEnabled) return;

    await SoundService().playMessageReceived();

    if (!kIsWeb) {
      try {
        const androidDetails = AndroidNotificationDetails(
          'bharatchat_updates',
          'BharatChat Updates',
          channelDescription: 'Over-The-Air app update notifications',
          importance: Importance.high,
          priority: Priority.high,
          showWhen: true,
          enableVibration: true,
          playSound: true,
        );

        const details = NotificationDetails(android: androidDetails);
        await _localNotifications.show(
          999,
          title,
          'Version $version is ready to download and install.',
          details,
        );
      } catch (e) {
        debugPrint('Failed to show update notification: $e');
      }
    }
  }

  /// Show incoming call notification
  Future<void> showCallNotification({
    required String callerName,
    required String callType,
  }) async {
    if (!isNotificationEnabled) return;

    if (!kIsWeb) {
      try {
        const androidDetails = AndroidNotificationDetails(
          'bharatchat_calls',
          'BharatChat Calls',
          channelDescription: 'Incoming voice and video call notifications',
          importance: Importance.max,
          priority: Priority.high,
          fullScreenIntent: true,
          ongoing: true,
          autoCancel: false,
          enableVibration: true,
          playSound: true,
        );

        const details = NotificationDetails(android: androidDetails);
        await _localNotifications.show(
          888,
          'Incoming $callType Call',
          '$callerName is calling...',
          details,
        );
      } catch (e) {
        debugPrint('Failed to show call notification: $e');
      }
    }
  }

  /// Cancel incoming call notification
  Future<void> cancelCallNotification() async {
    if (!kIsWeb) {
      try {
        await _localNotifications.cancel(888);
      } catch (_) {}
    }
  }
}

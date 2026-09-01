import 'dart:io';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:udaya_mobile/domain/entities/notification_entity.dart';
import 'package:udaya_mobile/services/firebase_options.dart' as udaya_firebase;

class NotificationService {
  static const String _channelId = 'udaya_notifications';
  static const String _channelName = 'Udaya Notifications';
  static const String _channelDescription = 'Notifications for Udaya wallet and network events';

  final FirebaseMessaging _firebaseMessaging = FirebaseMessaging.instance;
  final FlutterLocalNotificationsPlugin _localNotifications = FlutterLocalNotificationsPlugin();

  String? _fcmToken;
  final List<AppNotification> _notifications = [];
  AppNotificationSettings _settings = const AppNotificationSettings();

  String? get fcmToken => _fcmToken;
  List<AppNotification> get notifications => List.unmodifiable(_notifications);
  AppNotificationSettings get settings => _settings;

  Future<void> initialize() async {
    await Firebase.initializeApp(options: udaya_firebase.FirebaseOptions.currentPlatform);

    const androidSettings = AndroidInitializationSettings('@mipmap/ic_launcher');
    const iosSettings = DarwinInitializationSettings(
      requestAlertPermission: true,
      requestBadgePermission: true,
      requestSoundPermission: true,
    );
    const initSettings = InitializationSettings(android: androidSettings, iOS: iosSettings);

    await _localNotifications.initialize(initSettings);

    if (Platform.isAndroid) {
      await _createNotificationChannel();
    }

    await _requestPermissions();
    _fcmToken = await _firebaseMessaging.getToken();

    FirebaseMessaging.onMessage.listen(_onForegroundMessage);
    FirebaseMessaging.onMessageOpenedApp.listen(_onMessageOpened);
  }

  Future<void> _createNotificationChannel() async {
    const androidChannel = AndroidNotificationChannel(
      _channelId,
      _channelName,
      description: _channelDescription,
      importance: Importance.high,
      showBadge: true,
    );

    await _localNotifications
        .resolvePlatformSpecificImplementation<AndroidFlutterLocalNotificationsPlugin>()
        ?.createNotificationChannel(androidChannel);
  }

  Future<void> _requestPermissions() async {
    final settings = await _firebaseMessaging.requestPermission(
      alert: true,
      badge: true,
      sound: true,
    );

    if (settings.authorizationStatus == AuthorizationStatus.authorized) {
      await _localNotifications
          .resolvePlatformSpecificImplementation<IOSFlutterLocalNotificationsPlugin>()
          ?.requestPermissions(alert: true, badge: true, sound: true);
    }
  }

  Future<void> _onForegroundMessage(RemoteMessage message) async {
    final notification = AppNotification(
      id: message.messageId ?? DateTime.now().millisecondsSinceEpoch.toString(),
      type: _mapNotificationType(message.data['type']),
      title: message.notification?.title ?? 'Udaya Notification',
      body: message.notification?.body ?? '',
      payload: message.data.isNotEmpty ? Map.from(message.data) : null,
      timestamp: DateTime.now(),
    );

    _notifications.insert(0, notification);

    if (_settings.enabled && _settings.isTypeEnabled(notification.type.id)) {
      await _showLocalNotification(notification);
    }
  }

  Future<void> _onMessageOpened(RemoteMessage message) async {
    final notification = AppNotification(
      id: message.messageId ?? DateTime.now().millisecondsSinceEpoch.toString(),
      type: _mapNotificationType(message.data['type']),
      title: message.notification?.title ?? 'Udaya Notification',
      body: message.notification?.body ?? '',
      payload: message.data.isNotEmpty ? Map.from(message.data) : null,
      timestamp: DateTime.now(),
    );

    _notifications.insert(0, notification);
  }

  Future<void> _showLocalNotification(AppNotification notification) async {
    const androidDetails = AndroidNotificationDetails(
      _channelId,
      _channelName,
      channelDescription: _channelDescription,
      importance: Importance.high,
      priority: Priority.high,
      showWhen: true,
    );

    const iosDetails = DarwinNotificationDetails(
      presentAlert: true,
      presentBadge: true,
      presentSound: true,
    );

    const notificationDetails = NotificationDetails(
      android: androidDetails,
      iOS: iosDetails,
    );

    await _localNotifications.show(
      notification.id.hashCode,
      _settings.showPreview ? notification.title : 'Udaya',
      _settings.showPreview ? notification.body : 'New notification',
      notificationDetails,
      payload: notification.payload != null ? notification.payload.toString() : null,
    );
  }

  NotificationType _mapNotificationType(String? typeId) {
    return NotificationType.all.firstWhere(
      (type) => type.id == typeId,
      orElse: () => NotificationType.all[0],
    );
  }

  Future<void> updateSettings(AppNotificationSettings settings) async {
    _settings = settings;

    if (!settings.enabled) {
      await _localNotifications.cancelAll();
    }
  }

  Future<void> markAsRead(String notificationId) async {
    final index = _notifications.indexWhere((n) => n.id == notificationId);
    if (index != -1) {
      _notifications[index] = _notifications[index].copyWith(isRead: true);
    }
  }

  Future<void> clearAll() async {
    _notifications.clear();
    await _localNotifications.cancelAll();
  }

  Future<void> refreshToken() async {
    _fcmToken = await _firebaseMessaging.getToken();
  }
}

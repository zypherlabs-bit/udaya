import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/notification_entity.dart';
import 'package:udaya_mobile/services/notification_service.dart';

final notificationServiceProvider = Provider<NotificationService>((ref) {
  return NotificationService();
});

final notificationSettingsProvider = StateProvider<AppNotificationSettings>((ref) {
  return const AppNotificationSettings();
});

final notificationsProvider = Provider<List<AppNotification>>((ref) {
  final service = ref.watch(notificationServiceProvider);
  return service.notifications;
});

final unreadNotificationsCountProvider = Provider<int>((ref) {
  final notifications = ref.watch(notificationsProvider);
  return notifications.where((n) => !n.isRead).length;
});

final fcmTokenProvider = Provider<String?>((ref) {
  final service = ref.watch(notificationServiceProvider);
  return service.fcmToken;
});

final notificationEnabledProvider = Provider<bool>((ref) {
  final settings = ref.watch(notificationSettingsProvider);
  return settings.enabled;
});

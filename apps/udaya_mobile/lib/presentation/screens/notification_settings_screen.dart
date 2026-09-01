import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/notification_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/providers/notification_provider.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/status_chip.dart';

class NotificationSettingsScreen extends ConsumerStatefulWidget {
  const NotificationSettingsScreen({super.key});

  @override
  ConsumerState<NotificationSettingsScreen> createState() => _NotificationSettingsScreenState();
}

class _NotificationSettingsScreenState extends ConsumerState<NotificationSettingsScreen> {
  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(notificationSettingsProvider);
    final notifications = ref.watch(notificationsProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          final service = ref.read(notificationServiceProvider);
          await service.refreshToken();
          setState(() {});
        },
        child: CustomScrollView(
          slivers: [
            SliverAppBar(
              expandedHeight: 120,
              floating: false,
              pinned: true,
              backgroundColor: Colors.transparent,
              flexibleSpace: FlexibleSpaceBar(
                background: Container(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                      colors: [
                        UdayaTheme.bgDark.withValues(alpha: 0.95),
                        UdayaTheme.bgDark.withValues(alpha: 0.0),
                      ],
                    ),
                  ),
                ),
              ),
              title: Row(
                children: [
                  Container(
                    width: 32,
                    height: 32,
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        colors: [UdayaTheme.info, UdayaTheme.secondaryColor],
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Icon(Icons.notifications_rounded, color: UdayaTheme.textPrimary, size: 18),
                  ),
                  const SizedBox(width: 10),
                  const Text(
                    'Notifications',
                    style: TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.w600,
                      color: UdayaTheme.textPrimary,
                      letterSpacing: -0.3,
                    ),
                  ),
                ],
              ),
              actions: [
                if (notifications.isNotEmpty)
                  GlassIconButton(
                    icon: Icons.clear_all_rounded,
                    iconColor: UdayaTheme.textSecondary,
                    onPressed: () async {
                      final service = ref.read(notificationServiceProvider);
                      await service.clearAll();
                      setState(() {});
                    },
                  ),
              ],
            ),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _buildMasterToggle(settings),
                    const SizedBox(height: 16),
                    _buildNotificationList(notifications),
                    const SizedBox(height: 16),
                    _buildNotificationTypes(settings),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMasterToggle(AppNotificationSettings settings) {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: settings.enabled
                    ? [UdayaTheme.info.withValues(alpha: 0.2), UdayaTheme.secondaryColor.withValues(alpha: 0.1)]
                    : [UdayaTheme.textMuted.withValues(alpha: 0.1), UdayaTheme.textMuted.withValues(alpha: 0.05)],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Icon(
              settings.enabled ? Icons.notifications_active_rounded : Icons.notifications_off_rounded,
              color: settings.enabled ? UdayaTheme.info : UdayaTheme.textMuted,
              size: 22,
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Enable Notifications',
                  style: TextStyle(
                    color: UdayaTheme.textPrimary,
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                Text(
                  settings.enabled ? 'Notifications are enabled' : 'Notifications are disabled',
                  style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13),
                ),
              ],
            ),
          ),
          Switch(
            value: settings.enabled,
            onChanged: (value) {
              ref.read(notificationSettingsProvider.notifier).state = settings.copyWith(enabled: value);
              ref.read(notificationServiceProvider).updateSettings(
                ref.read(notificationSettingsProvider),
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildNotificationList(List<AppNotification> notifications) {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Recent Notifications',
                style: TextStyle(
                  color: UdayaTheme.textPrimary,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
              if (notifications.isNotEmpty)
                StatusChip(
                  label: '${notifications.length} total',
                  color: UdayaTheme.info,
                ),
            ],
          ),
          const SizedBox(height: 12),
          if (notifications.isEmpty)
            Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  children: [
                    Icon(Icons.notifications_none_rounded, size: 40, color: UdayaTheme.textMuted.withValues(alpha: 0.5)),
                    const SizedBox(height: 12),
                    Text(
                      'No notifications yet',
                      style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 14),
                    ),
                  ],
                ),
              ),
            )
          else
            ...notifications.take(10).map((notification) {
              return _NotificationTile(
                notification: notification,
                onTap: () {
                  ref.read(notificationServiceProvider).markAsRead(notification.id);
                  setState(() {});
                },
              );
            }),
        ],
      ),
    );
  }

  Widget _buildNotificationTypes(AppNotificationSettings settings) {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Notification Types',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          ...NotificationType.all.map((type) {
            final isEnabled = settings.isTypeEnabled(type.id);
            final icon = _getTypeIcon(type.id);
            final color = _getTypeColor(type.id);
            return Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: Row(
                children: [
                  Container(
                    width: 36,
                    height: 36,
                    decoration: BoxDecoration(
                      color: isEnabled ? color.withValues(alpha: 0.12) : UdayaTheme.bgSurface,
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Icon(icon, color: isEnabled ? color : UdayaTheme.textMuted, size: 18),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          type.title,
                          style: TextStyle(
                            color: UdayaTheme.textPrimary,
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        Text(
                          type.description,
                          style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  Switch(
                    value: isEnabled,
                    onChanged: (value) {
                      final newTypeSettings = Map<String, bool>.from(settings.typeSettings);
                      newTypeSettings[type.id] = value;
                      ref.read(notificationSettingsProvider.notifier).state = settings.copyWith(
                        typeSettings: newTypeSettings,
                      );
                      ref.read(notificationServiceProvider).updateSettings(
                        ref.read(notificationSettingsProvider),
                      );
                    },
                  ),
                ],
              ),
            );
          }),
        ],
      ),
    );
  }

  Color _getTypeColor(String typeId) {
    switch (typeId) {
      case 'transaction_confirmed':
        return UdayaTheme.confirmedColor;
      case 'node_status_change':
        return UdayaTheme.info;
      case 'peer_connected':
        return UdayaTheme.success;
      case 'peer_disconnected':
        return UdayaTheme.error;
      case 'block_found':
        return UdayaTheme.warning;
      case 'security_alert':
        return UdayaTheme.error;
      default:
        return UdayaTheme.secondaryColor;
    }
  }

  IconData _getTypeIcon(String typeId) {
    switch (typeId) {
      case 'transaction_confirmed':
        return Icons.receipt_long_rounded;
      case 'node_status_change':
        return Icons.dns_rounded;
      case 'peer_connected':
        return Icons.people_rounded;
      case 'peer_disconnected':
        return Icons.person_remove_rounded;
      case 'block_found':
        return Icons.memory_rounded;
      case 'security_alert':
        return Icons.security_rounded;
      default:
        return Icons.notifications_rounded;
    }
  }
}

class _NotificationTile extends StatelessWidget {
  final AppNotification notification;
  final VoidCallback onTap;

  const _NotificationTile({required this.notification, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(10),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: _getTypeColor(notification.type.id).withValues(alpha: 0.12),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(
                notification.isRead ? Icons.notifications_none_rounded : Icons.notifications_active_rounded,
                color: notification.isRead ? UdayaTheme.textTertiary : _getTypeColor(notification.type.id),
                size: 18,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    notification.title,
                    style: TextStyle(
                      color: notification.isRead ? UdayaTheme.textSecondary : UdayaTheme.textPrimary,
                      fontSize: 14,
                      fontWeight: notification.isRead ? FontWeight.normal : FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    notification.body,
                    style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 8),
            Text(
              _formatTime(notification.timestamp),
              style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 11),
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);
    if (diff.inSeconds < 60) return '${diff.inSeconds}s';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m';
    if (diff.inHours < 24) return '${diff.inHours}h';
    return '${diff.inDays}d';
  }

  Color _getTypeColor(String typeId) {
    switch (typeId) {
      case 'transaction_confirmed':
        return UdayaTheme.confirmedColor;
      case 'node_status_change':
        return UdayaTheme.info;
      case 'peer_connected':
        return UdayaTheme.success;
      case 'peer_disconnected':
        return UdayaTheme.error;
      case 'block_found':
        return UdayaTheme.warning;
      case 'security_alert':
        return UdayaTheme.error;
      default:
        return UdayaTheme.secondaryColor;
    }
  }
}

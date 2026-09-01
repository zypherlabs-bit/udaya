import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class ConfirmDialog extends StatelessWidget {
  final String title;
  final String message;
  final String confirmText;
  final String cancelText;
  final bool isDestructive;
  final IconData? icon;

  const ConfirmDialog({
    super.key,
    required this.title,
    required this.message,
    this.confirmText = 'Confirm',
    this.cancelText = 'Cancel',
    this.isDestructive = false,
    this.icon,
  });

  static Future<bool?> show(
    BuildContext context, {
    required String title,
    required String message,
    String confirmText = 'Confirm',
    String cancelText = 'Cancel',
    bool isDestructive = false,
    IconData? icon,
  }) {
    return showDialog<bool>(
      context: context,
      builder: (context) => ConfirmDialog(
        title: title,
        message: message,
        confirmText: confirmText,
        cancelText: cancelText,
        isDestructive: isDestructive,
        icon: icon,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: UdayaTheme.bgCard,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      title: Row(
        children: [
          if (icon != null) ...[
            Icon(
              icon,
              color: isDestructive ? UdayaTheme.error : UdayaTheme.info,
            ),
            const SizedBox(width: 12),
          ],
          Text(
            title,
            style: const TextStyle(color: UdayaTheme.textPrimary),
          ),
        ],
      ),
      content: Text(
        message,
        style: const TextStyle(color: UdayaTheme.textSecondary),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: Text(cancelText),
        ),
        ElevatedButton(
          onPressed: () => Navigator.of(context).pop(true),
          style: isDestructive
              ? ElevatedButton.styleFrom(backgroundColor: UdayaTheme.error)
              : null,
          child: Text(confirmText),
        ),
      ],
    );
  }
}

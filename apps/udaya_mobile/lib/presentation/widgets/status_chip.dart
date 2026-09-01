import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class StatusChip extends StatelessWidget {
  final String label;
  final Color color;
  final IconData? icon;

  const StatusChip({
    super.key,
    required this.label,
    required this.color,
    this.icon,
  });

  factory StatusChip.success(String label) => StatusChip(
        label: label,
        color: UdayaTheme.success,
        icon: Icons.check_circle,
      );

  factory StatusChip.warning(String label) => StatusChip(
        label: label,
        color: UdayaTheme.warning,
        icon: Icons.warning,
      );

  factory StatusChip.error(String label) => StatusChip(
        label: label,
        color: UdayaTheme.error,
        icon: Icons.error,
      );

  factory StatusChip.info(String label) => StatusChip(
        label: label,
        color: UdayaTheme.info,
        icon: Icons.info,
      );

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (icon != null) ...[
            Icon(icon, size: 14, color: color),
            const SizedBox(width: 4),
          ],
          Text(
            label,
            style: TextStyle(
              color: color,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}

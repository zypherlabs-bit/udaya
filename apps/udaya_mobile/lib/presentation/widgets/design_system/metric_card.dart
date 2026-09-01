import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'glass_card.dart';
import 'animated_number.dart';

enum TransactionCardStatus { confirmed, pending, failed }

class MetricCard extends StatelessWidget {
  final String label;
  final String value;
  final IconData? icon;
  final Color? iconColor;
  final Color? glowColor;
  final String? subValue;
  final bool animate;
  final Duration? animationDuration;
  final VoidCallback? onTap;

  const MetricCard({
    super.key,
    required this.label,
    required this.value,
    this.icon,
    this.iconColor,
    this.glowColor,
    this.subValue,
    this.animate = false,
    this.animationDuration,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveIconColor = iconColor ?? UdayaTheme.primaryColor;
    final effectiveGlow = glowColor ?? UdayaTheme.primaryGlow;

    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(16),
      animationDuration: animationDuration,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (icon != null) ...[
            Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: effectiveIconColor.withValues(alpha: 0.15),
                borderRadius: BorderRadius.circular(10),
              ),
              child: Icon(icon, color: effectiveIconColor, size: 18),
            ),
            const SizedBox(height: 12),
          ],
          AnimatedNumber(
            value: value,
            style: const TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
              letterSpacing: -0.5,
              height: 1.3,
            ),
            animate: animate,
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: const TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w500,
              color: UdayaTheme.textTertiary,
              letterSpacing: 0.3,
            ),
          ),
          if (subValue != null) ...[
            const SizedBox(height: 4),
            Text(
              subValue!,
              style: const TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w400,
                color: UdayaTheme.textMuted,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class TransactionCard extends StatelessWidget {
  final String txid;
  final String amount;
  final String? address;
  final TransactionCardStatus status;
  final int timestamp;
  final int confirmations;
  final String fee;
  final int? blockHeight;
  final VoidCallback? onTap;

  const TransactionCard({
    super.key,
    required this.txid,
    required this.amount,
    this.address,
    required this.status,
    required this.timestamp,
    required this.confirmations,
    required this.fee,
    this.blockHeight,
    this.onTap,
  });

  Color get statusColor {
    switch (status) {
      case TransactionCardStatus.confirmed:
        return UdayaTheme.confirmedColor;
      case TransactionCardStatus.pending:
        return UdayaTheme.syncingColor;
      case TransactionCardStatus.failed:
        return UdayaTheme.error;
    }
  }

  String get statusLabel {
    switch (status) {
      case TransactionCardStatus.confirmed:
        return 'CONFIRMED';
      case TransactionCardStatus.pending:
        return 'PENDING';
      case TransactionCardStatus.failed:
        return 'FAILED';
    }
  }

  @override
  Widget build(BuildContext context) {
    final dateTime = DateTime.fromMillisecondsSinceEpoch(timestamp * 1000);
    final dateStr = '${dateTime.day.toString().padLeft(2, '0')}.${dateTime.month.toString().padLeft(2, '0')}.${dateTime.year}';

    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                  border: Border.all(color: statusColor.withValues(alpha: 0.3), width: 1),
                ),
                child: Text(
                  statusLabel,
                  style: TextStyle(
                    color: statusColor,
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
              const Spacer(),
              Text(
                dateStr,
                style: const TextStyle(
                  color: UdayaTheme.textTertiary,
                  fontSize: 12,
                  fontWeight: FontWeight.w400,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      amount,
                      style: const TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                        letterSpacing: -0.3,
                        height: 1.3,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      fee != '0' ? 'Fee: $fee UDAYA' : 'No fee',
                      style: const TextStyle(
                        fontSize: 12,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
              if (status == TransactionStatus.confirmed && confirmations > 0)
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      '$confirmations',
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w700,
                        color: UdayaTheme.confirmedColor,
                        letterSpacing: -0.5,
                      ),
                    ),
                    const Text(
                      'confirmations',
                      style: TextStyle(
                        fontSize: 11,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
            ],
          ),
          if (address != null && address!.isNotEmpty) ...[
            const SizedBox(height: 10),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: UdayaTheme.dividerColor, width: 1),
              ),
              child: Row(
                children: [
                  const Icon(Icons.arrow_outward_rounded, size: 14, color: UdayaTheme.textTertiary),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      address!,
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                        color: UdayaTheme.textSecondary,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
          ],
          const SizedBox(height: 10),
          Row(
            children: [
              Icon(Icons.fingerprint_rounded, size: 12, color: UdayaTheme.textMuted),
              const SizedBox(width: 6),
              Expanded(
                child: Text(
                  txid,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 11,
                    color: UdayaTheme.textMuted,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (blockHeight != null) ...[
                const SizedBox(width: 12),
                Text(
                  'Block #$blockHeight',
                  style: const TextStyle(
                    fontSize: 11,
                    color: UdayaTheme.textMuted,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }
}

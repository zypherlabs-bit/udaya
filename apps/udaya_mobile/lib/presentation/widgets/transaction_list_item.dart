import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/extensions/extensions.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';

class TransactionListItem extends StatelessWidget {
  final WalletTransaction transaction;
  final VoidCallback? onTap;

  const TransactionListItem({
    super.key,
    required this.transaction,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final isReceived = transaction.direction == TransactionDirection.received;
    final isPending = transaction.status == TransactionStatus.pending;

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        onTap: onTap,
        leading: CircleAvatar(
          backgroundColor: _getIconColor(isReceived, isPending).withValues(alpha: 0.2),
          child: Icon(
            _getIcon(isReceived, transaction.status),
            color: _getIconColor(isReceived, isPending),
            size: 22,
          ),
        ),
        title: Text(
          isReceived ? 'Received' : 'Sent',
          style: const TextStyle(
            color: UdayaTheme.textPrimary,
            fontWeight: FontWeight.w600,
          ),
        ),
        subtitle: Text(
          DateTime.fromMillisecondsSinceEpoch(transaction.timestamp * 1000).toRelativeTime,
          style: const TextStyle(color: UdayaTheme.textTertiary),
        ),
        trailing: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Text(
              '${isReceived ? '+' : '-'}${transaction.amountUdya.toStringAsFixed(8)} UDAYA',
              style: TextStyle(
                color: isReceived ? UdayaTheme.success : UdayaTheme.error,
                fontWeight: FontWeight.w600,
                fontSize: 14,
              ),
            ),
            if (isPending)
              Container(
                margin: const EdgeInsets.only(top: 4),
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: UdayaTheme.warning.withValues(alpha: 0.2),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: const Text(
                  'PENDING',
                  style: TextStyle(
                    color: UdayaTheme.warning,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              )
            else
              Text(
                '${transaction.confirmations} conf.',
                style: const TextStyle(
                  color: UdayaTheme.textTertiary,
                  fontSize: 12,
                ),
              ),
          ],
        ),
      ),
    );
  }

  IconData _getIcon(bool isReceived, TransactionStatus status) {
    if (status == TransactionStatus.pending) return Icons.access_time;
    return isReceived ? Icons.arrow_downward : Icons.arrow_upward;
  }

  Color _getIconColor(bool isReceived, bool isPending) {
    if (isPending) return UdayaTheme.warning;
    return isReceived ? UdayaTheme.success : UdayaTheme.error;
  }
}

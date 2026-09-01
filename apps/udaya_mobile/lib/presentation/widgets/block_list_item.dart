import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/extensions/extensions.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';

class BlockListItem extends StatelessWidget {
  final BlockInfo block;
  final VoidCallback? onTap;

  const BlockListItem({
    super.key,
    required this.block,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        onTap: onTap,
        leading: CircleAvatar(
          backgroundColor: UdayaTheme.primaryColor.withValues(alpha: 0.2),
          child: const Icon(
            Icons.widgets,
            color: UdayaTheme.secondaryColor,
            size: 22,
          ),
        ),
        title: Text(
          'Block #${block.height}',
          style: const TextStyle(
            color: UdayaTheme.textPrimary,
            fontWeight: FontWeight.w600,
          ),
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              block.hash.truncateMiddle,
              style: const TextStyle(
                color: UdayaTheme.textTertiary,
                fontSize: 12,
              ),
            ),
            const SizedBox(height: 2),
            Text(
              '${block.transactionCount} txs | ${(block.size / 1024).toStringAsFixed(1)} KB',
              style: const TextStyle(
                color: UdayaTheme.textTertiary,
                fontSize: 12,
              ),
            ),
          ],
        ),
        trailing: Text(
          DateTime.fromMillisecondsSinceEpoch(block.timestamp * 1000).toRelativeTime,
          style: const TextStyle(
            color: UdayaTheme.textSecondary,
            fontSize: 12,
          ),
        ),
      ),
    );
  }
}

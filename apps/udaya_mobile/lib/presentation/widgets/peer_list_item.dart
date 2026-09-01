import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';
import 'package:udaya_mobile/core/extensions/extensions.dart';

class PeerListItem extends StatelessWidget {
  final PeerInfo peer;
  final VoidCallback? onTap;
  final VoidCallback? onDisconnect;

  const PeerListItem({
    super.key,
    required this.peer,
    this.onTap,
    this.onDisconnect,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        onTap: onTap,
        leading: Container(
          width: 12,
          height: 12,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: peer.isConnected ? UdayaTheme.success : UdayaTheme.error,
          ),
        ),
        title: Text(
          peer.address.truncateMiddle,
          style: const TextStyle(
            color: UdayaTheme.textPrimary,
            fontWeight: FontWeight.w500,
          ),
        ),
        subtitle: Text(
          'Height: ${peer.height} | ${peer.pingTimeMs}ms | ${peer.isOutbound ? "Outbound" : "Inbound"}',
          style: const TextStyle(color: UdayaTheme.textTertiary),
        ),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: peer.isOutbound
                    ? UdayaTheme.info.withValues(alpha: 0.2)
                    : UdayaTheme.secondaryColor.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                peer.isOutbound ? 'OUT' : 'IN',
                style: TextStyle(
                  color: peer.isOutbound ? UdayaTheme.info : UdayaTheme.secondaryColor,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
            if (onDisconnect != null) ...[
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.close, size: 18),
                color: UdayaTheme.error,
                onPressed: onDisconnect,
                tooltip: 'Disconnect peer',
              ),
            ],
          ],
        ),
      ),
    );
  }
}

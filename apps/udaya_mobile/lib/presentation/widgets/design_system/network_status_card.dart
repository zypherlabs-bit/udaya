import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'glass_card.dart';
import 'metric_card.dart';
import 'status_indicator.dart';

class NetworkStatusCard extends StatelessWidget {
  final String network;
  final int peerCount;
  final bool isSyncing;
  final int syncPercentage;
  final int blockHeight;
  final int? networkHeight;
  final VoidCallback? onTap;

  const NetworkStatusCard({
    super.key,
    required this.network,
    required this.peerCount,
    required this.isSyncing,
    required this.syncPercentage,
    required this.blockHeight,
    this.networkHeight,
    this.onTap,
  });

  Color get networkColor {
    switch (network.toLowerCase()) {
      case 'testnet':
        return UdayaTheme.testnetColor;
      case 'regtest':
        return UdayaTheme.regtestColor;
      default:
        return UdayaTheme.mainnetColor;
    }
  }

  String get networkLabel {
    switch (network.toLowerCase()) {
      case 'testnet':
        return 'TESTNET';
      case 'regtest':
        return 'REGTEST';
      default:
        return 'MAINNET';
    }
  }

  @override
  Widget build(BuildContext context) {
    final effectiveNetworkColor = networkColor;

    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: isSyncing ? UdayaTheme.syncingColor : effectiveNetworkColor,
                size: 10,
                pulse: isSyncing,
              ),
              const SizedBox(width: 10),
              Text(
                networkLabel,
                style: TextStyle(
                  color: effectiveNetworkColor,
                  fontSize: 13,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 1,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                decoration: BoxDecoration(
                  color: effectiveNetworkColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                  border: Border.all(color: effectiveNetworkColor.withValues(alpha: 0.25), width: 1),
                ),
                child: Text(
                  isSyncing ? 'SYNCING' : 'SYNCED',
                  style: TextStyle(
                    color: effectiveNetworkColor,
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Block Height',
                  value: blockHeight.toString(),
                  icon: Icons.public_rounded,
                  iconColor: UdayaTheme.secondaryColor,
                  subValue: isSyncing ? 'Syncing...' : 'Synced',
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Peers',
                  value: peerCount.toString(),
                  icon: Icons.people_rounded,
                  iconColor: UdayaTheme.accentColor,
                  subValue: 'Connected',
                ),
              ),
            ],
          ),
          if (networkHeight != null) ...[
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: MetricCard(
                    label: 'Network Height',
                    value: networkHeight.toString(),
                    icon: Icons.public_rounded,
                    iconColor: UdayaTheme.info,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: MetricCard(
                    label: 'Sync Progress',
                    value: '$syncPercentage%',
                    icon: Icons.sync_rounded,
                    iconColor: isSyncing ? UdayaTheme.syncingColor : UdayaTheme.confirmedColor,
                  ),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }
}

class NodeStatusCard extends StatelessWidget {
  final String status;
  final int blockHeight;
  final String chainwork;
  final double difficulty;
  final int peerCount;
  final int syncPercentage;
  final DateTime? lastBlockTime;
  final String network;
  final double cpuUsage;
  final double memoryUsage;
  final int diskUsageBytes;
  final VoidCallback? onTap;

  const NodeStatusCard({
    super.key,
    required this.status,
    required this.blockHeight,
    required this.chainwork,
    required this.difficulty,
    required this.peerCount,
    required this.syncPercentage,
    this.lastBlockTime,
    required this.network,
    required this.cpuUsage,
    required this.memoryUsage,
    required this.diskUsageBytes,
    this.onTap,
  });

  Color get statusColor {
    switch (status.toLowerCase()) {
      case 'running':
        return UdayaTheme.confirmedColor;
      case 'starting':
      case 'stopping':
        return UdayaTheme.syncingColor;
      case 'error':
        return UdayaTheme.error;
      default:
        return UdayaTheme.textMuted;
    }
  }

  String get statusLabel => status.toUpperCase();

  @override
  Widget build(BuildContext context) {
    final diskMB = (diskUsageBytes / (1024 * 1024)).toStringAsFixed(1);

    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: statusColor,
                size: 10,
                pulse: status == 'running',
              ),
              const SizedBox(width: 10),
              Text(
                'Node Status',
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.2,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                  border: Border.all(color: statusColor.withValues(alpha: 0.25), width: 1),
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
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Block Height',
                  value: blockHeight.toString(),
                  icon: Icons.public_rounded,
                  iconColor: UdayaTheme.secondaryColor,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Peers',
                  value: peerCount.toString(),
                  icon: Icons.people_rounded,
                  iconColor: UdayaTheme.accentColor,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'CPU',
                  value: '${cpuUsage.toStringAsFixed(1)}%',
                  icon: Icons.memory_rounded,
                  iconColor: UdayaTheme.warning,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Memory',
                  value: '${memoryUsage.toStringAsFixed(0)} MB',
                  icon: Icons.storage_rounded,
                  iconColor: UdayaTheme.info,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Storage',
                  value: '$diskMB MB',
                  icon: Icons.folder_rounded,
                  iconColor: UdayaTheme.primaryColor,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Network',
                  value: network.toUpperCase(),
                  icon: Icons.public_rounded,
                  iconColor: UdayaTheme.success,
                ),
              ),
            ],
          ),
          if (chainwork.isNotEmpty) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: UdayaTheme.dividerColor, width: 1),
              ),
              child: Row(
                children: [
                  const Icon(Icons.link_rounded, size: 14, color: UdayaTheme.textTertiary),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Chainwork: $chainwork',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 11,
                        color: UdayaTheme.textSecondary,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

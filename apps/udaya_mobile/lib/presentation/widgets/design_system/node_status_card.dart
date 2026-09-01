import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'glass_card.dart';
import 'metric_card.dart';
import 'status_indicator.dart';

class PeerCard extends StatelessWidget {
  final String peerId;
  final int height;
  final int latency;
  final String direction;
  final bool isConnected;
  final String? protocolVersion;
  final VoidCallback? onTap;

  const PeerCard({
    super.key,
    required this.peerId,
    required this.height,
    required this.latency,
    required this.direction,
    required this.isConnected,
    this.protocolVersion,
    this.onTap,
  });

  Color get latencyColor {
    if (latency < 100) return UdayaTheme.confirmedColor;
    if (latency < 300) return UdayaTheme.syncingColor;
    return UdayaTheme.error;
  }

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(14),
      child: Row(
        children: [
          StatusIndicator(
            color: isConnected ? UdayaTheme.confirmedColor : UdayaTheme.error,
            size: 8,
            pulse: isConnected,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  peerId,
                  style: const TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w500,
                    color: UdayaTheme.textPrimary,
                    fontFamily: 'monospace',
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 2),
                Text(
                  'Height: $height  |  ${direction.toUpperCase()}  |  ${latency}ms',
                  style: const TextStyle(
                    fontSize: 11,
                    color: UdayaTheme.textTertiary,
                  ),
                ),
              ],
            ),
          ),
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: latencyColor,
              shape: BoxShape.circle,
              boxShadow: [
                BoxShadow(
                  color: latencyColor.withValues(alpha: 0.4),
                  blurRadius: 8,
                  spreadRadius: 2,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class MiningCard extends StatelessWidget {
  final bool isMining;
  final double hashrate;
  final double difficulty;
  final int blocksFound;
  final int blockReward;
  final String miningAddress;
  final int threadCount;
  final double? cpuUsage;
  final VoidCallback? onToggle;

  const MiningCard({
    super.key,
    required this.isMining,
    required this.hashrate,
    required this.difficulty,
    required this.blocksFound,
    required this.blockReward,
    required this.miningAddress,
    required this.threadCount,
    this.cpuUsage,
    this.onToggle,
  });

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: isMining ? UdayaTheme.confirmedColor : UdayaTheme.textMuted,
                size: 10,
                pulse: isMining,
              ),
              const SizedBox(width: 10),
              Text(
                'Mining',
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
                  color: (isMining ? UdayaTheme.confirmedColor : UdayaTheme.textMuted).withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                  border: Border.all(
                    color: (isMining ? UdayaTheme.confirmedColor : UdayaTheme.textMuted).withValues(alpha: 0.25),
                    width: 1,
                  ),
                ),
                child: Text(
                  isMining ? 'ACTIVE' : 'INACTIVE',
                  style: TextStyle(
                    color: isMining ? UdayaTheme.confirmedColor : UdayaTheme.textMuted,
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
                  label: 'Hashrate',
                  value: '${hashrate.toStringAsFixed(1)} H/s',
                  icon: Icons.speed_rounded,
                  iconColor: UdayaTheme.info,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Blocks Found',
                  value: blocksFound.toString(),
                  icon: Icons.public_rounded,
                  iconColor: UdayaTheme.success,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Difficulty',
                  value: difficulty.toStringAsFixed(0),
                  icon: Icons.trending_up_rounded,
                  iconColor: UdayaTheme.warning,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Block Reward',
                  value: '${(blockReward / 100000000).toStringAsFixed(2)} UDAYA',
                  icon: Icons.card_giftcard_rounded,
                  iconColor: UdayaTheme.accentColor,
                ),
              ),
            ],
          ),
          if (cpuUsage != null) ...[
            const SizedBox(height: 12),
            MetricCard(
              label: 'CPU Usage',
              value: '${cpuUsage!.toStringAsFixed(1)}%',
              icon: Icons.memory_rounded,
              iconColor: UdayaTheme.warning,
            ),
          ],
          const SizedBox(height: 16),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: UdayaTheme.dividerColor, width: 1),
            ),
            child: Row(
              children: [
                const Icon(Icons.pin_drop_rounded, size: 14, color: UdayaTheme.textTertiary),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Mining Address: $miningAddress',
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
      ),
    );
  }
}

class SecurityCard extends StatelessWidget {
  final String title;
  final String status;
  final IconData icon;
  final Color? iconColor;
  final Color? statusColor;
  final String? description;
  final VoidCallback? onTap;

  const SecurityCard({
    super.key,
    required this.title,
    required this.status,
    required this.icon,
    this.iconColor,
    this.statusColor,
    this.description,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveIconColor = iconColor ?? UdayaTheme.confirmedColor;
    final effectiveStatusColor = statusColor ?? UdayaTheme.confirmedColor;

    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: effectiveIconColor.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(icon, color: effectiveIconColor, size: 20),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: UdayaTheme.textPrimary,
                  ),
                ),
                if (description != null) ...[
                  const SizedBox(height: 2),
                  Text(
                    description!,
                    style: const TextStyle(
                      fontSize: 12,
                      color: UdayaTheme.textTertiary,
                    ),
                  ),
                ],
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
            decoration: BoxDecoration(
              color: effectiveStatusColor.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
              border: Border.all(color: effectiveStatusColor.withValues(alpha: 0.25), width: 1),
            ),
            child: Text(
              status,
              style: TextStyle(
                color: effectiveStatusColor,
                fontSize: 11,
                fontWeight: FontWeight.w600,
                letterSpacing: 0.5,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

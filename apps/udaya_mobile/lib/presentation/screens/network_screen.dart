import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';

class NetworkScreen extends ConsumerWidget {
  const NetworkScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final blockchainState = ref.watch(blockchainProvider);
    final nodeState = ref.watch(nodeProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: blockchainState.isLoading
          ? const LoadingIndicator(message: 'Loading network data...')
          : blockchainState.error != null
              ? ErrorDisplay(message: blockchainState.error!)
              : RefreshIndicator(
                  onRefresh: () async {
                    await ref.read(blockchainProvider.notifier).refresh();
                  },
                  child: CustomScrollView(
                    slivers: [
                      const SliverAppBar(
                        pinned: true,
                        floating: false,
                        backgroundColor: Colors.transparent,
                        elevation: 0,
                        title: Text(
                          'Network',
                          style: TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                            color: UdayaTheme.textPrimary,
                            letterSpacing: -0.3,
                          ),
                        ),
                      ),
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              _buildNetworkStats(nodeState, blockchainState),
                              const SizedBox(height: 16),
                              _buildPeerCountChart(blockchainState),
                              const SizedBox(height: 16),
                              _buildBlockTimeDistribution(),
                              const SizedBox(height: 16),
                              _buildFeeEstimation(),
                            ],
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
    );
  }

  Widget _buildNetworkStats(NodeState nodeState, BlockchainState blockchainState) {
    final status = nodeState.status;

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Network Statistics',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          GridView.count(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            crossAxisCount: 2,
            childAspectRatio: 2.5,
            crossAxisSpacing: 12,
            mainAxisSpacing: 12,
            children: [
              _StatBox(
                label: 'Block Height',
                value: status?.blockHeight.toString() ?? '0',
              ),
              _StatBox(
                label: 'Difficulty',
                value: status?.difficulty.toStringAsFixed(2) ?? '0',
              ),
              _StatBox(
                label: 'Hashrate',
                value: '${((status?.difficulty ?? 0) / 600 / 1e6).toStringAsFixed(2)} MH/s',
              ),
              _StatBox(
                label: 'Peer Count',
                value: blockchainState.peers.length.toString(),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildPeerCountChart(BlockchainState blockchainState) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Peer Connections',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          SizedBox(
            height: 150,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              crossAxisAlignment: CrossAxisAlignment.end,
              children: List.generate(7, (index) {
                final height = (20 + (index * 15) % 80).toDouble();
                return Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Container(
                      width: 24,
                      height: height,
                      decoration: BoxDecoration(
                        color: UdayaTheme.secondaryColor.withValues(alpha: 0.7),
                        borderRadius: BorderRadius.circular(4),
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'D${index + 1}',
                      style: const TextStyle(
                        color: UdayaTheme.textTertiary,
                        fontSize: 10,
                      ),
                    ),
                  ],
                );
              }),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBlockTimeDistribution() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Block Time Distribution',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _TimeBar(label: '< 5 min', percentage: 0.3, color: UdayaTheme.success),
          _TimeBar(label: '5-10 min', percentage: 0.45, color: UdayaTheme.info),
          _TimeBar(label: '10-15 min', percentage: 0.2, color: UdayaTheme.warning),
          _TimeBar(label: '> 15 min', percentage: 0.05, color: UdayaTheme.error),
        ],
      ),
    );
  }

  Widget _buildFeeEstimation() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Fee Estimation',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _FeeRow(label: 'High Priority', feeRate: '15 sat/vB', time: '~5 min'),
          _FeeRow(label: 'Medium Priority', feeRate: '8 sat/vB', time: '~15 min'),
          _FeeRow(label: 'Low Priority', feeRate: '3 sat/vB', time: '~30 min'),
          _FeeRow(label: 'Economy', feeRate: '1 sat/vB', time: '~60 min'),
        ],
      ),
    );
  }
}

class _StatBox extends StatelessWidget {
  final String label;
  final String value;

  const _StatBox({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      padding: const EdgeInsets.all(12),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            value,
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
          ),
          Text(
            label,
            style: const TextStyle(
              color: UdayaTheme.textTertiary,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}

class _TimeBar extends StatelessWidget {
  final String label;
  final double percentage;
  final Color color;

  const _TimeBar({
    required this.label,
    required this.percentage,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 12),
            ),
          ),
          Expanded(
            child: ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: percentage,
                backgroundColor: UdayaTheme.bgSurface,
                valueColor: AlwaysStoppedAnimation(color),
                minHeight: 8,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Text(
            '${(percentage * 100).toInt()}%',
            style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 12),
          ),
        ],
      ),
    );
  }
}

class _FeeRow extends StatelessWidget {
  final String label;
  final String feeRate;
  final String time;

  const _FeeRow({
    required this.label,
    required this.feeRate,
    required this.time,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: const TextStyle(color: UdayaTheme.textSecondary),
          ),
          Row(
            children: [
              Text(
                feeRate,
                style: const TextStyle(
                  color: UdayaTheme.textPrimary,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(width: 12),
              Text(
                time,
                style: const TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

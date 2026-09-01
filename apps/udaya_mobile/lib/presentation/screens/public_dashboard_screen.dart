import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/node_entity.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';
import 'package:udaya_mobile/domain/entities/decentralization_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/status_chip.dart';
import 'package:udaya_mobile/presentation/widgets/dashboard_charts.dart';
import 'package:udaya_mobile/services/dashboard_service.dart';

class PublicDashboardScreen extends ConsumerWidget {
  const PublicDashboardScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final nodeAsync = ref.watch(nodeStatusProvider);
    final peersAsync = ref.watch(peersProvider);
    final mempoolAsync = ref.watch(mempoolInfoProvider);
    final miningAsync = ref.watch(miningInfoProvider);
    final decentralizationAsync = ref.watch(decentralizationMetricsProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(nodeStatusProvider);
          ref.invalidate(peersProvider);
          ref.invalidate(mempoolInfoProvider);
          ref.invalidate(miningInfoProvider);
          ref.invalidate(decentralizationMetricsProvider);
        },
        child: CustomScrollView(
          slivers: [
            const SliverAppBar(
              pinned: true,
              floating: false,
              backgroundColor: Colors.transparent,
              elevation: 0,
              title: Text(
                'Network Dashboard',
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
                    _buildSummaryCard(ref, nodeAsync, miningAsync, mempoolAsync, decentralizationAsync),
                    const SizedBox(height: 16),
                    _buildAlertsCard(ref, nodeAsync, miningAsync, decentralizationAsync),
                    const SizedBox(height: 16),
                    _buildHashrateChart(ref, nodeAsync, miningAsync),
                    const SizedBox(height: 16),
                    _buildBlockTimeChart(ref, nodeAsync),
                    const SizedBox(height: 16),
                    _buildPeerDistributionChart(ref, decentralizationAsync, peersAsync),
                    const SizedBox(height: 16),
                    _buildMempoolChart(ref, mempoolAsync),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryCard(
    WidgetRef ref,
    AsyncValue<NodeStatus> nodeAsync,
    AsyncValue<MiningInfo> miningAsync,
    AsyncValue<MempoolInfo> mempoolAsync,
    AsyncValue<DecentralizationMetrics> decentralizationAsync,
  ) {
    final nodeStatus = nodeAsync.value;
    final miningInfo = miningAsync.value;
    final mempoolInfo = mempoolAsync.value;
    final diversity = decentralizationAsync.value?.peerDiversity;

    final dashboardService = DashboardService();
    final summary = dashboardService.generateSummary(
      nodeStatus: nodeStatus,
      miningInfo: miningInfo,
      mempoolInfo: mempoolInfo,
      peerDiversity: diversity,
    );

    final healthColor = summary.networkHealthScore >= 0.8
        ? UdayaTheme.success
        : summary.networkHealthScore >= 0.5
            ? UdayaTheme.warning
            : UdayaTheme.error;

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Network Summary',
                style: TextStyle(
                  color: UdayaTheme.textPrimary,
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                ),
              ),
              StatusChip(
                label: '${(summary.networkHealthScore * 100).toInt()}% Healthy',
                color: healthColor,
              ),
            ],
          ),
          const SizedBox(height: 16),
          _buildStatRow('Block Height', summary.currentBlockHeight.toString()),
          _buildStatRow('Hashrate', '${summary.currentHashrate.toStringAsFixed(2)} H/s'),
          _buildStatRow('Active Peers', summary.activePeers.toString()),
          _buildStatRow('Difficulty', summary.currentDifficulty.toStringAsFixed(2)),
          _buildStatRow('Avg Fee Rate', '${summary.avgFeeRate.toStringAsFixed(2)} sat/byte'),
          _buildStatRow('Mempool Size', '${summary.totalTransactions} txns'),
          const SizedBox(height: 12),
          Text(
            'Last updated: ${_formatTime(summary.lastUpdated)}',
            style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
          ),
        ],
      ),
    );
  }

  Widget _buildAlertsCard(
    WidgetRef ref,
    AsyncValue<NodeStatus> nodeAsync,
    AsyncValue<MiningInfo> miningAsync,
    AsyncValue<DecentralizationMetrics> decentralizationAsync,
  ) {
    final dashboardService = DashboardService();
    final alerts = dashboardService.generateAlerts(
      nodeStatus: nodeAsync.value,
      miningInfo: miningAsync.value,
      peerDiversity: decentralizationAsync.value?.peerDiversity,
    );

    if (alerts.isEmpty) {
      return GlassCard(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(Icons.check_circle, color: UdayaTheme.success, size: 20),
            const SizedBox(width: 8),
            Text(
              'No alerts',
              style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14),
            ),
          ],
        ),
      );
    }

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Alerts',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          ...alerts.map((alert) => Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Row(
              children: [
                Icon(
                  alert.severity == 'critical' ? Icons.error : Icons.warning,
                  size: 16,
                  color: alert.severity == 'critical' ? UdayaTheme.error : UdayaTheme.warning,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    alert.message,
                    style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13),
                  ),
                ),
              ],
            ),
          )),
        ],
      ),
    );
  }

  Widget _buildHashrateChart(
    WidgetRef ref,
    AsyncValue<NodeStatus> nodeAsync,
    AsyncValue<MiningInfo> miningAsync,
  ) {
    final dashboardService = DashboardService();
    final peers = ref.read(peersProvider).value ?? [];
    final blocks = ref.read(blockchainProvider).blocks;

    final timeSeries = dashboardService.generateTimeSeries(
      blocks: blocks,
      peers: peers,
      miningInfo: miningAsync.value,
    );

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Network Hashrate',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          HashrateChart(
            timestamps: timeSeries.timestamps,
            hashrates: timeSeries.hashrates,
          ),
        ],
      ),
    );
  }

  Widget _buildBlockTimeChart(WidgetRef ref, AsyncValue<NodeStatus> nodeAsync) {
    final dashboardService = DashboardService();
    final blocks = ref.read(blockchainProvider).blocks;
    final blockTimeData = dashboardService.generateBlockTimeData(blocks);

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Block Time Distribution',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          BlockTimeChart(
            timestamps: blockTimeData.timestamps,
            blockTimes: blockTimeData.blockTimes,
          ),
        ],
      ),
    );
  }

  Widget _buildPeerDistributionChart(
    WidgetRef ref,
    AsyncValue<DecentralizationMetrics> decentralizationAsync,
    AsyncValue<List<PeerInfo>> peersAsync,
  ) {
    final dashboardService = DashboardService();
    final peerDistribution = dashboardService.generatePeerDistribution(
      peers: peersAsync.value ?? [],
      diversityMetrics: decentralizationAsync.value?.peerDiversity,
    );

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Peer Distribution',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          PeerDistributionChart(
            topCountries: peerDistribution.topCountries,
            topAsns: peerDistribution.topAsns,
          ),
        ],
      ),
    );
  }

  Widget _buildMempoolChart(WidgetRef ref, AsyncValue<MempoolInfo> mempoolAsync) {
    final dashboardService = DashboardService();
    final mempoolStats = dashboardService.generateMempoolStats(
      mempoolAsync.value ?? MempoolInfo(
        size: 0,
        bytes: 0,
        maxMempool: 0,
        mempoolMinFee: 0,
        minRelayTxFee: 0,
      ),
    );

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Mempool Fee Distribution',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          MempoolChart(feeDistribution: mempoolStats.feeDistribution),
        ],
      ),
    );
  }

  Widget _buildStatRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14)),
          Text(
            value,
            style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14, fontWeight: FontWeight.w600),
          ),
        ],
      ),
    );
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);
    if (diff.inSeconds < 60) return '${diff.inSeconds}s ago';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    return '${diff.inHours}h ago';
  }
}

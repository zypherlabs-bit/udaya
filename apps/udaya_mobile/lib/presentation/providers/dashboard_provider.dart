import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/dashboard_entity.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/services/dashboard_service.dart';

final dashboardServiceProvider = Provider<DashboardService>((ref) {
  return DashboardService();
});

final dashboardSummaryProvider = Provider<DashboardSummary>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final nodeStatus = ref.watch(nodeStatusProvider).value;
  final miningInfo = ref.watch(miningInfoProvider).value;
  final mempoolInfo = ref.watch(mempoolInfoProvider).value;
  final decentralization = ref.watch(decentralizationMetricsProvider).value;

  return service.generateSummary(
    nodeStatus: nodeStatus,
    miningInfo: miningInfo,
    mempoolInfo: mempoolInfo,
    peerDiversity: decentralization?.peerDiversity,
  );
});

final dashboardTimeSeriesProvider = Provider<NetworkStatsTimeSeries>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final blocks = ref.watch(blockchainProvider).blocks;
  final peers = ref.watch(peersProvider).value ?? [];
  final miningInfo = ref.watch(miningInfoProvider).value;

  return service.generateTimeSeries(
    blocks: blocks,
    peers: peers,
    miningInfo: miningInfo,
  );
});

final dashboardPeerDistributionProvider = Provider<PeerDistributionData>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final peers = ref.watch(peersProvider).value ?? [];
  final decentralization = ref.watch(decentralizationMetricsProvider).value;

  return service.generatePeerDistribution(
    peers: peers,
    diversityMetrics: decentralization?.peerDiversity,
  );
});

final dashboardBlockTimeProvider = Provider<BlockTimeData>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final blocks = ref.watch(blockchainProvider).blocks;
  return service.generateBlockTimeData(blocks);
});

final dashboardMempoolStatsProvider = Provider<MempoolStatsData>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final mempool = ref.watch(mempoolInfoProvider).value ??
      MempoolInfo(
        size: 0,
        bytes: 0,
        maxMempool: 0,
        mempoolMinFee: 0,
        minRelayTxFee: 0,
      );
  return service.generateMempoolStats(mempool);
});

final dashboardAlertsProvider = Provider<List<DashboardAlert>>((ref) {
  final service = ref.watch(dashboardServiceProvider);
  final nodeStatus = ref.watch(nodeStatusProvider).value;
  final miningInfo = ref.watch(miningInfoProvider).value;
  final decentralization = ref.watch(decentralizationMetricsProvider).value;

  return service.generateAlerts(
    nodeStatus: nodeStatus,
    miningInfo: miningInfo,
    peerDiversity: decentralization?.peerDiversity,
  );
});

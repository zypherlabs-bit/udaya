import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/decentralization_entity.dart';
import 'package:udaya_mobile/presentation/providers/node_provider.dart';
import 'package:udaya_mobile/services/decentralization_service.dart';

final decentralizationServiceProvider = Provider<DecentralizationService>((ref) {
  return DecentralizationService();
});

final decentralizationMetricsProvider = FutureProvider<DecentralizationMetrics>((ref) async {
  final service = ref.watch(decentralizationServiceProvider);
  final peers = ref.watch(peersProvider).value ?? [];

  final totalNodes = peers.length;
  final activeNodes = peers.where((p) => p.isConnected).length;

  return service.calculateDecentralizationMetrics(
    peers,
    totalNodes: totalNodes,
    activeNodes: activeNodes,
  );
});

final regionDistributionProvider = FutureProvider<List<RegionInfo>>((ref) async {
  final service = ref.watch(decentralizationServiceProvider);
  final peers = ref.watch(peersProvider).value ?? [];
  return service.getRegionDistribution(peers);
});

final decentralizationAlertsProvider = FutureProvider<List<DecentralizationAlert>>((ref) async {
  final metrics = ref.watch(decentralizationMetricsProvider).value;
  if (metrics == null) return [];

  final service = ref.watch(decentralizationServiceProvider);
  return service.checkDecentralizationAlerts(metrics);
});

final nakamotoCoefficientProvider = Provider<NakamotoCoefficient>((ref) {
  final metrics = ref.watch(decentralizationMetricsProvider).value;
  return metrics?.nakamotoCoefficient ??
      NakamotoCoefficient(
        value: 0,
        entityType: 'ASN',
        entityPercentages: {},
        calculatedAt: DateTime.now(),
      );
});

final peerDiversityProvider = Provider<PeerDiversityMetrics>((ref) {
  final metrics = ref.watch(decentralizationMetricsProvider).value;
  return metrics?.peerDiversity ??
      PeerDiversityMetrics(
        totalPeers: 0,
        uniqueCountries: 0,
        uniqueAsns: 0,
        uniqueProviders: 0,
        countryDistribution: {},
        asnDistribution: {},
        providerDistribution: {},
        diversityScore: 0.0,
      );
});

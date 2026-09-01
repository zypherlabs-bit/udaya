import 'dart:math';
import 'package:udaya_mobile/domain/entities/decentralization_entity.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';

class DecentralizationService {
  static const int _targetNakamotoCoefficient = 5;

  PeerDiversityMetrics calculatePeerDiversity(List<PeerInfo> peers) {
    final countryDistribution = <String, int>{};
    final asnDistribution = <String, int>{};
    final providerDistribution = <String, int>{};

    for (final peer in peers) {
      final parts = peer.address.split('.');
      final countryCode = _estimateCountryFromIp(parts);
      final asn = _estimateAsnFromIp(parts);
      final provider = _estimateProviderFromIp(parts);

      countryDistribution[countryCode] = (countryDistribution[countryCode] ?? 0) + 1;
      asnDistribution[asn] = (asnDistribution[asn] ?? 0) + 1;
      providerDistribution[provider] = (providerDistribution[provider] ?? 0) + 1;
    }

    final uniqueCountries = countryDistribution.length;
    final uniqueAsns = asnDistribution.length;
    final uniqueProviders = providerDistribution.length;
    final totalPeers = peers.length;

    final diversityScore = _calculateDiversityScore(
      totalPeers,
      uniqueCountries,
      uniqueAsns,
      uniqueProviders,
    );

    return PeerDiversityMetrics(
      totalPeers: totalPeers,
      uniqueCountries: uniqueCountries,
      uniqueAsns: uniqueAsns,
      uniqueProviders: uniqueProviders,
      countryDistribution: countryDistribution,
      asnDistribution: asnDistribution,
      providerDistribution: providerDistribution,
      diversityScore: diversityScore,
    );
  }

  NakamotoCoefficient calculateNakamotoCoefficient(List<PeerInfo> peers) {
    if (peers.isEmpty) {
      return NakamotoCoefficient(
        value: 0,
        entityType: 'ASN',
        entityPercentages: {},
        calculatedAt: DateTime.now(),
      );
    }

    final asnDistribution = <String, int>{};
    for (final peer in peers) {
      final parts = peer.address.split('.');
      final asn = _estimateAsnFromIp(parts);
      asnDistribution[asn] = (asnDistribution[asn] ?? 0) + 1;
    }

    final sortedEntities = asnDistribution.entries.toList()
      ..sort((a, b) => b.value.compareTo(a.value));

    final total = peers.length;
    final percentages = <String, double>{};
    for (final entry in sortedEntities) {
      percentages[entry.key] = entry.value / total;
    }

    int cumulative = 0;
    int nakamotoValue = 0;
    for (final entry in sortedEntities) {
      cumulative += entry.value;
      nakamotoValue++;
      if (cumulative >= total / 2) break;
    }

    return NakamotoCoefficient(
      value: max(nakamotoValue, 1),
      entityType: 'ASN',
      entityPercentages: percentages,
      calculatedAt: DateTime.now(),
    );
  }

  DecentralizationMetrics calculateDecentralizationMetrics(
    List<PeerInfo> peers, {
    required int totalNodes,
    required int activeNodes,
  }) {
    final peerDiversity = calculatePeerDiversity(peers);
    final nakamoto = calculateNakamotoCoefficient(peers);

    final decentralizationScore = _calculateOverallScore(
      peerDiversity.diversityScore,
      nakamoto.value,
      totalNodes,
      activeNodes,
    );

    return DecentralizationMetrics(
      peerDiversity: peerDiversity,
      nakamotoCoefficient: nakamoto,
      totalNodes: totalNodes,
      activeNodes: activeNodes,
      decentralizationScore: decentralizationScore,
    );
  }

  List<RegionInfo> getRegionDistribution(List<PeerInfo> peers) {
    final regionCounts = <String, int>{};
    final regionNames = <String, String>{
      'US': 'United States',
      'EU': 'Europe',
      'AP': 'Asia Pacific',
      'SA': 'South America',
      'AF': 'Africa',
      'OC': 'Oceania',
    };

    for (final peer in peers) {
      final parts = peer.address.split('.');
      final region = _estimateRegionFromIp(parts);
      regionCounts[region] = (regionCounts[region] ?? 0) + 1;
    }

    final total = peers.length;
    return regionCounts.entries.map((entry) {
      return RegionInfo(
        code: entry.key,
        name: regionNames[entry.key] ?? entry.key,
        peerCount: entry.value,
        percentage: total > 0 ? entry.value / total : 0.0,
      );
    }).toList()..sort((a, b) => b.percentage.compareTo(a.percentage));
  }

  List<DecentralizationAlert> checkDecentralizationAlerts(
    DecentralizationMetrics metrics,
  ) {
    final alerts = <DecentralizationAlert>[];

    if (metrics.peerDiversity.uniqueAsns < _targetNakamotoCoefficient) {
      alerts.add(DecentralizationAlert(
        id: 'low_nakamoto',
        severity: 'warning',
        message: 'Nakamoto coefficient ${metrics.nakamotoCoefficient.value} is below target $_targetNakamotoCoefficient',
        timestamp: DateTime.now(),
      ));
    }

    if (metrics.peerDiversity.uniqueCountries < 3) {
      alerts.add(DecentralizationAlert(
        id: 'low_geographic',
        severity: 'warning',
        message: 'Only ${metrics.peerDiversity.uniqueCountries} countries represented',
        timestamp: DateTime.now(),
      ));
    }

    if (metrics.peerDiversity.diversityScore < 0.5) {
      alerts.add(DecentralizationAlert(
        id: 'low_diversity',
        severity: 'critical',
        message: 'Network diversity score is critically low',
        timestamp: DateTime.now(),
      ));
    }

    return alerts;
  }

  double _calculateDiversityScore(
    int totalPeers,
    int uniqueCountries,
    int uniqueAsns,
    int uniqueProviders,
  ) {
    if (totalPeers == 0) return 0.0;

    final countryScore = min(uniqueCountries / 5.0, 1.0);
    final asnScore = min(uniqueAsns / _targetNakamotoCoefficient.toDouble(), 1.0);
    final providerScore = min(uniqueProviders / 3.0, 1.0);

    return (countryScore * 0.4 + asnScore * 0.4 + providerScore * 0.2);
  }

  double _calculateOverallScore(
    double diversityScore,
    int nakamotoValue,
    int totalNodes,
    int activeNodes,
  ) {
    final nakamotoScore = min(nakamotoValue / _targetNakamotoCoefficient.toDouble(), 1.0);
    final nodeHealthScore = totalNodes > 0 ? activeNodes / totalNodes : 0.0;

    return diversityScore * 0.5 + nakamotoScore * 0.3 + nodeHealthScore * 0.2;
  }

  String _estimateCountryFromIp(List<String> parts) {
    if (parts.isEmpty) return 'XX';
    final firstOctet = int.tryParse(parts[0]) ?? 0;

    if (firstOctet >= 1 && firstOctet <= 126) return 'US';
    if (firstOctet >= 128 && firstOctet <= 191) return 'EU';
    if (firstOctet >= 192 && firstOctet <= 223) return 'AP';

    return 'XX';
  }

  String _estimateAsnFromIp(List<String> parts) {
    if (parts.isEmpty) return 'AS0';
    final hash = parts.join('').hashCode;
    return 'AS${(hash.abs() % 65535).toString().padLeft(5, '0')}';
  }

  String _estimateProviderFromIp(List<String> parts) {
    if (parts.isEmpty) return 'Unknown';
    final hash = parts.join('').hashCode;
    final providers = ['Cloud', 'ISP', 'Datacenter', 'Mobile'];
    return providers[hash.abs() % providers.length];
  }

  String _estimateRegionFromIp(List<String> parts) {
    if (parts.isEmpty) return 'XX';
    final firstOctet = int.tryParse(parts[0]) ?? 0;

    if (firstOctet >= 1 && firstOctet <= 126) return 'US';
    if (firstOctet >= 128 && firstOctet <= 191) return 'EU';
    if (firstOctet >= 192 && firstOctet <= 223) return 'AP';

    return 'XX';
  }
}

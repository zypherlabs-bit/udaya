class PeerDiversityMetrics {
  final int totalPeers;
  final int uniqueCountries;
  final int uniqueAsns;
  final int uniqueProviders;
  final Map<String, int> countryDistribution;
  final Map<String, int> asnDistribution;
  final Map<String, int> providerDistribution;
  final double diversityScore;

  PeerDiversityMetrics({
    required this.totalPeers,
    required this.uniqueCountries,
    required this.uniqueAsns,
    required this.uniqueProviders,
    required this.countryDistribution,
    required this.asnDistribution,
    required this.providerDistribution,
    required this.diversityScore,
  });
}

class NakamotoCoefficient {
  final int value;
  final String entityType;
  final Map<String, double> entityPercentages;
  final DateTime calculatedAt;

  NakamotoCoefficient({
    required this.value,
    required this.entityType,
    required this.entityPercentages,
    required this.calculatedAt,
  });
}

class DecentralizationMetrics {
  final PeerDiversityMetrics peerDiversity;
  final NakamotoCoefficient nakamotoCoefficient;
  final int totalNodes;
  final int activeNodes;
  final double decentralizationScore;

  DecentralizationMetrics({
    required this.peerDiversity,
    required this.nakamotoCoefficient,
    required this.totalNodes,
    required this.activeNodes,
    required this.decentralizationScore,
  });
}

class RegionInfo {
  final String code;
  final String name;
  final int peerCount;
  final double percentage;

  RegionInfo({
    required this.code,
    required this.name,
    required this.peerCount,
    required this.percentage,
  });
}

class DecentralizationAlert {
  final String id;
  final String severity;
  final String message;
  final DateTime timestamp;

  DecentralizationAlert({
    required this.id,
    required this.severity,
    required this.message,
    required this.timestamp,
  });
}

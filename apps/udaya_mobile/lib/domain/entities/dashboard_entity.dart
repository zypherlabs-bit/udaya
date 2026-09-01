class NetworkStatsTimeSeries {
  final List<DateTime> timestamps;
  final List<double> hashrates;
  final List<int> blockHeights;
  final List<int> peerCounts;
  final List<double> difficulties;
  final List<double> feeRates;

  NetworkStatsTimeSeries({
    required this.timestamps,
    required this.hashrates,
    required this.blockHeights,
    required this.peerCounts,
    required this.difficulties,
    required this.feeRates,
  });
}

class DashboardSummary {
  final int currentBlockHeight;
  final double currentHashrate;
  final int activePeers;
  final double currentDifficulty;
  final double avgFeeRate;
  final int totalTransactions;
  final double networkHealthScore;
  final DateTime lastUpdated;

  DashboardSummary({
    required this.currentBlockHeight,
    required this.currentHashrate,
    required this.activePeers,
    required this.currentDifficulty,
    required this.avgFeeRate,
    required this.totalTransactions,
    required this.networkHealthScore,
    required this.lastUpdated,
  });
}

class PeerDistributionData {
  final Map<String, int> countryDistribution;
  final Map<String, int> asnDistribution;
  final Map<String, int> providerDistribution;
  final List<MapEntry<String, int>> topCountries;
  final List<MapEntry<String, int>> topAsns;

  PeerDistributionData({
    required this.countryDistribution,
    required this.asnDistribution,
    required this.providerDistribution,
    required this.topCountries,
    required this.topAsns,
  });
}

class BlockTimeData {
  final List<DateTime> timestamps;
  final List<int> blockTimes; // seconds between blocks

  BlockTimeData({
    required this.timestamps,
    required this.blockTimes,
  });
}

class MempoolStatsData {
  final int size;
  final int bytes;
  final double minFeeRate;
  final int maxMempool;
  final List<MapEntry<int, int>> feeDistribution;

  MempoolStatsData({
    required this.size,
    required this.bytes,
    required this.minFeeRate,
    required this.maxMempool,
    required this.feeDistribution,
  });
}

class DashboardAlert {
  final String id;
  final String severity;
  final String message;
  final DateTime timestamp;

  DashboardAlert({
    required this.id,
    required this.severity,
    required this.message,
    required this.timestamp,
  });
}

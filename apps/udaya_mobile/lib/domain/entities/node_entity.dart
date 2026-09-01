class NetworkInfo {
  final String network; // mainnet, testnet, regtest
  final int peerCount;
  final int connectionsIn;
  final int connectionsOut;
  final String version;
  final String subVersion;
  final int protocolVersion;
  final bool networkActive;

  NetworkInfo({
    required this.network,
    required this.peerCount,
    required this.connectionsIn,
    required this.connectionsOut,
    required this.version,
    required this.subVersion,
    required this.protocolVersion,
    required this.networkActive,
  });
}

class NodeStatus {
  final String status; // stopped, starting, running, stopping, error
  final int blockHeight;
  final String bestBlockHash;
  final String chainwork;
  final double difficulty;
  final int peerCount;
  final int syncPercentage;
  final DateTime? lastBlockTime;
  final String network;
  final double cpuUsage;
  final double memoryUsage;
  final int diskUsageBytes;

  NodeStatus({
    required this.status,
    required this.blockHeight,
    required this.bestBlockHash,
    required this.chainwork,
    required this.difficulty,
    required this.peerCount,
    required this.syncPercentage,
    this.lastBlockTime,
    required this.network,
    required this.cpuUsage,
    required this.memoryUsage,
    required this.diskUsageBytes,
  });

  NodeStatus copyWith({
    String? status,
    int? blockHeight,
    String? bestBlockHash,
    String? chainwork,
    double? difficulty,
    int? peerCount,
    int? syncPercentage,
    DateTime? lastBlockTime,
    String? network,
    double? cpuUsage,
    double? memoryUsage,
    int? diskUsageBytes,
  }) {
    return NodeStatus(
      status: status ?? this.status,
      blockHeight: blockHeight ?? this.blockHeight,
      bestBlockHash: bestBlockHash ?? this.bestBlockHash,
      chainwork: chainwork ?? this.chainwork,
      difficulty: difficulty ?? this.difficulty,
      peerCount: peerCount ?? this.peerCount,
      syncPercentage: syncPercentage ?? this.syncPercentage,
      lastBlockTime: lastBlockTime ?? this.lastBlockTime,
      network: network ?? this.network,
      cpuUsage: cpuUsage ?? this.cpuUsage,
      memoryUsage: memoryUsage ?? this.memoryUsage,
      diskUsageBytes: diskUsageBytes ?? this.diskUsageBytes,
    );
  }
}

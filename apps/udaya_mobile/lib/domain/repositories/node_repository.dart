import '../entities/node_entity.dart';
import '../entities/blockchain_entity.dart';

abstract class NodeRepository {
  Future<NodeStatus> getNodeStatus();
  Future<NetworkInfo> getNetworkInfo();
  Future<int> getBlockHeight();
  Future<String> getBestBlockHash();
  Future<String> getChainwork();
  Future<int> getPeerCount();
  Future<List<PeerInfo>> getPeers();
  Future<int> startNode({String? configPath, bool? enableMining, int? minerThreads});
  Future<int> stopNode();
  Future<bool> isNodeRunning();
  Future<MempoolInfo> getMempoolInfo();
  Stream<NodeStatus> getNodeStatusStream();
}

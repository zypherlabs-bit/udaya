import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/entities/node_entity.dart';
import '../../domain/entities/blockchain_entity.dart';
import '../../domain/repositories/node_repository.dart';
import '../../domain/repositories/mining_repository.dart';
import '../../domain/repositories/blockchain_repository.dart';

/// Provider for the node repository.
final nodeRepositoryProvider = Provider<NodeRepository>((ref) {
  throw UnimplementedError('nodeRepositoryProvider must be overridden');
});

/// Provider for the mining repository.
final miningRepositoryProvider = Provider<MiningRepository>((ref) {
  throw UnimplementedError('miningRepositoryProvider must be overridden');
});

/// Provider for the blockchain repository.
final blockchainRepositoryProvider = Provider<BlockchainRepository>((ref) {
  throw UnimplementedError('blockchainRepositoryProvider must be overridden');
});

/// Provider for the current node status.
///
/// Automatically refreshes when dependencies change.
final nodeStatusProvider = FutureProvider<NodeStatus>((ref) async {
  final repo = ref.watch(nodeRepositoryProvider);
  return repo.getNodeStatus();
});

/// Provider for network information.
final networkInfoProvider = FutureProvider<NetworkInfo>((ref) async {
  final repo = ref.watch(nodeRepositoryProvider);
  return repo.getNetworkInfo();
});

/// Provider for connected peers list.
final peersProvider = FutureProvider<List<PeerInfo>>((ref) async {
  final repo = ref.watch(nodeRepositoryProvider);
  return repo.getPeers();
});

/// Provider for mempool information.
final mempoolInfoProvider = FutureProvider<MempoolInfo>((ref) async {
  final repo = ref.watch(nodeRepositoryProvider);
  return repo.getMempoolInfo();
});

/// Provider for mining information.
final miningInfoProvider = FutureProvider<MiningInfo>((ref) async {
  final repo = ref.watch(miningRepositoryProvider);
  return repo.getMiningInfo();
});

/// Stream provider that emits node status updates in real-time.
final nodeStatusStreamProvider = StreamProvider<NodeStatus>((ref) {
  final repo = ref.watch(nodeRepositoryProvider);
  return repo.getNodeStatusStream();
});

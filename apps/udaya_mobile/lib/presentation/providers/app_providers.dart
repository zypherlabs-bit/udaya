import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'package:udaya_mobile/domain/entities/node_entity.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';
import 'package:udaya_mobile/data/datasources/remote/udaya_rpc_client.dart';
import 'package:udaya_mobile/domain/repositories/node_repository.dart';
import 'package:udaya_mobile/domain/repositories/blockchain_repository.dart';
import 'package:udaya_mobile/presentation/providers/node_provider.dart';
import 'package:udaya_mobile/data/datasources/local/shared_prefs_source.dart';
export 'wallet_provider.dart';
export 'monitoring_provider.dart';
export 'decentralization_provider.dart';
export 'node_provider.dart';
export 'security_provider.dart';
export 'exchange_provider.dart';
export 'dashboard_provider.dart';
export 'notification_provider.dart';

enum AppNetwork { mainnet, testnet, regtest }

class NetworkState {
  final AppNetwork network;
  final bool isSyncing;

  NetworkState({required this.network, this.isSyncing = false});

  NetworkState copyWith({AppNetwork? network, bool? isSyncing}) {
    return NetworkState(
      network: network ?? this.network,
      isSyncing: isSyncing ?? this.isSyncing,
    );
  }
}

class NetworkNotifier extends StateNotifier<NetworkState> {
  NetworkNotifier(this._prefs) : super(NetworkState(network: _prefs.getSelectedNetwork().contains('mainnet') ? AppNetwork.mainnet : _prefs.getSelectedNetwork().contains('regtest') ? AppNetwork.regtest : AppNetwork.testnet));

  final SharedPrefsSource _prefs;

  Future<void> setNetwork(AppNetwork network) async {
    final networkString = network == AppNetwork.mainnet ? 'mainnet' : network == AppNetwork.testnet ? 'testnet' : 'regtest';
    await _prefs.setSelectedNetwork(networkString);
    state = state.copyWith(network: network);
  }

  void setSyncing(bool syncing) {
    state = state.copyWith(isSyncing: syncing);
  }
}

final networkProvider = StateNotifierProvider<NetworkNotifier, NetworkState>((ref) {
  final prefs = ref.watch(sharedPrefsSourceProvider);
  return NetworkNotifier(prefs);
});

final sharedPrefsSourceProvider = Provider<SharedPrefsSource>((ref) {
  throw UnimplementedError('sharedPrefsSourceProvider must be overridden');
});

class NodeState {
  final NodeStatus? status;
  final bool isLoading;
  final String? error;

  NodeState({this.status, this.isLoading = false, this.error});
}

class NodeNotifier extends StateNotifier<NodeState> {
  NodeNotifier(this._repo) : super(NodeState(isLoading: true)) {
    _loadStatus();
  }

  final NodeRepository _repo;

  Future<void> _loadStatus() async {
    try {
      final status = await _repo.getNodeStatus();
      state = NodeState(status: status);
    } on UdayaRpcException {
      state = NodeState(
        status: NodeStatus(
          status: 'unavailable',
          blockHeight: 0,
          bestBlockHash: '',
          chainwork: '',
          difficulty: 0.0,
          peerCount: 0,
          syncPercentage: 0,
          lastBlockTime: null,
          network: 'mainnet',
          cpuUsage: 0.0,
          memoryUsage: 0.0,
          diskUsageBytes: 0,
        ),
        error: 'Udaya node unavailable — Retry',
      );
    } catch (e) {
      state = NodeState(
        status: NodeStatus(
          status: 'error',
          blockHeight: 0,
          bestBlockHash: '',
          chainwork: '',
          difficulty: 0.0,
          peerCount: 0,
          syncPercentage: 0,
          lastBlockTime: null,
          network: 'mainnet',
          cpuUsage: 0.0,
          memoryUsage: 0.0,
          diskUsageBytes: 0,
        ),
        error: 'Udaya node unavailable — Retry',
      );
    }
  }

  void updateStatus(NodeStatus status) {
    state = NodeState(status: status);
  }

  void setLoading(bool loading) {
    state = NodeState(status: state.status, isLoading: loading);
    if (loading) _loadStatus();
  }

  void setError(String error) {
    state = NodeState(status: state.status, error: error);
  }
}

final nodeProvider = StateNotifierProvider<NodeNotifier, NodeState>((ref) {
  final repo = ref.watch(nodeRepositoryProvider);
  return NodeNotifier(repo);
});

class BlockchainState {
  final List<BlockInfo> blocks;
  final List<PeerInfo> peers;
  final List<WalletTransaction> mempoolTxs;
  final bool isLoading;
  final String? error;

  BlockchainState({
    this.blocks = const [],
    this.peers = const [],
    this.mempoolTxs = const [],
    this.isLoading = false,
    this.error,
  });
}

class BlockchainNotifier extends StateNotifier<BlockchainState> {
  BlockchainNotifier(this._blockchainRepo, this._nodeRepo)
      : super(BlockchainState(isLoading: true)) {
    _loadInitialData();
  }

  final BlockchainRepository _blockchainRepo;
  final NodeRepository _nodeRepo;

  Future<void> _loadInitialData() async {
    try {
      final blocks = await _blockchainRepo.getLatestBlocks(limit: 20);
      final peers = await _nodeRepo.getPeers();
      final mempoolInfo = await _nodeRepo.getMempoolInfo();

      if (mempoolInfo.size > 0) {
        final rawMempool = await _blockchainRepo.getRawMempool();
        final mempoolTxs = rawMempool.take(10).map(_toWalletTransaction).toList();
        state = BlockchainState(
          blocks: blocks,
          peers: peers,
          mempoolTxs: mempoolTxs,
        );
      } else {
        state = BlockchainState(
          blocks: blocks,
          peers: peers,
          mempoolTxs: const [],
        );
      }
    } on UdayaRpcException catch (e) {
      state = BlockchainState(
        blocks: const [],
        peers: const [],
        mempoolTxs: const [],
        error: 'Udaya node unavailable — Retry',
      );
    } catch (e) {
      state = BlockchainState(
        blocks: const [],
        peers: const [],
        mempoolTxs: const [],
        error: 'Udaya node unavailable — Retry',
      );
    }
  }

  WalletTransaction _toWalletTransaction(TransactionInfo tx) {
    final totalOutput = tx.outputs.fold<int>(0, (sum, out) => sum + out.value);
    return WalletTransaction(
      txid: tx.txid,
      timestamp: tx.timestamp,
      direction: TransactionDirection.received,
      status: tx.confirmations > 0 ? TransactionStatus.confirmed : TransactionStatus.pending,
      amount: totalOutput,
      fee: tx.fee,
      confirmations: tx.confirmations,
      blockHeight: tx.blockHeight,
      address: tx.outputs.isNotEmpty ? tx.outputs.first.address : null,
      label: null,
    );
  }

  Future<void> refresh() async {
    state = BlockchainState(
      blocks: state.blocks,
      peers: state.peers,
      mempoolTxs: state.mempoolTxs,
      isLoading: true,
      error: null,
    );
    await _loadInitialData();
  }

  void updateBlocks(List<BlockInfo> blocks) {
    state = BlockchainState(
      blocks: blocks,
      peers: state.peers,
      mempoolTxs: state.mempoolTxs,
    );
  }

  void updatePeers(List<PeerInfo> peers) {
    state = BlockchainState(
      blocks: state.blocks,
      peers: peers,
      mempoolTxs: state.mempoolTxs,
    );
  }

  void updateMempool(List<WalletTransaction> txs) {
    state = BlockchainState(
      blocks: state.blocks,
      peers: state.peers,
      mempoolTxs: txs,
    );
  }
}

final blockchainProvider = StateNotifierProvider<BlockchainNotifier, BlockchainState>((ref) {
  final repo = ref.watch(blockchainRepositoryProvider);
  final nodeRepo = ref.watch(nodeRepositoryProvider);
  return BlockchainNotifier(repo, nodeRepo);
});

class MiningState {
  final MiningInfo? info;
  final bool isLoading;

  MiningState({this.info, this.isLoading = false});
}

class MiningNotifier extends StateNotifier<MiningState> {
  MiningNotifier() : super(MiningState(isLoading: true));

  void updateInfo(MiningInfo info) {
    state = MiningState(info: info);
  }

  void setLoading(bool loading) {
    state = MiningState(info: state.info, isLoading: loading);
  }
}

final miningProvider = StateNotifierProvider<MiningNotifier, MiningState>((ref) {
  return MiningNotifier();
});

final selectedTransactionProvider = StateProvider<WalletTransaction?>((ref) => null);
final selectedBlockProvider = StateProvider<BlockInfo?>((ref) => null);
final selectedPeerProvider = StateProvider<PeerInfo?>((ref) => null);

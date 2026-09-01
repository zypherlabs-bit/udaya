import '../entities/blockchain_entity.dart';

abstract class BlockchainRepository {
  Future<BlockInfo?> getBlock({int? height, String? hash});
  Future<List<BlockInfo>> getLatestBlocks({int? limit});
  Future<TransactionInfo?> getTransaction({required String txid});
  Future<List<TransactionInfo>> getBlockTransactions({required String blockHash});
  Future<List<TransactionInfo>> getRawMempool();
  Future<BlockInfo?> getBlockByHeight(int height);
  Future<String> getBlockHash(int height);
  Future<double> getNetworkHashrate();
  Future<double> getDifficulty();
  Future<int> getEstimatedFeeRate();
}

import '../entities/blockchain_entity.dart';

abstract class MiningRepository {
  Future<MiningInfo> getMiningInfo();
  Future<int> startMining({required int threads, String? address});
  Future<int> stopMining();
  Future<bool> isMining();
  Future<String> getMiningAddress();
  Future<int> getBlocksFound();
}

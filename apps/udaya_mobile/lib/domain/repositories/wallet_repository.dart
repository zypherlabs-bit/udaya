import '../entities/wallet_entity.dart';

abstract class WalletRepository {
  Future<String> createWallet({required String name, required String passphrase, String? network});
  Future<String> restoreWallet({required String seedPhrase, required String passphrase, String? network});
  Future<String> generateAddress({int? accountIndex});
  Future<List<String>> getAddresses({int? accountIndex});
  Future<WalletBalance> getBalance();
  Future<List<WalletTransaction>> getTransactions({int? limit, int? offset});
  Future<List<WalletUtxo>> getUtxos();
  Future<String> sendTransaction({required String address, required int amountSats, required int feeRate});
  Future<String> signMessage({required String message, required String address});
  Future<bool> verifyMessage({required String message, required String signature, required String address});
  Future<bool> isWalletCreated();
  Future<String> exportSeedPhrase({String? passphrase});
  Future<bool> lockWallet();
  Future<bool> unlockWallet({required String passphrase});
  Future<bool> isLocked();
  Future<void> backupWallet({required String destinationPath});
}

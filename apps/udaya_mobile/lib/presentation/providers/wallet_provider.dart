import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'package:udaya_mobile/domain/repositories/wallet_repository.dart';

final walletRepositoryProvider = Provider<WalletRepository>((ref) {
  throw UnimplementedError('walletRepositoryProvider must be overridden');
});

class WalletState {
  final WalletBalance? balance;
  final List<WalletAccount> accounts;
  final List<WalletTransaction> transactions;
  final bool isLoading;
  final String? error;

  WalletState({
    this.balance,
    this.accounts = const [],
    this.transactions = const [],
    this.isLoading = false,
    this.error,
  });
}

class WalletNotifier extends StateNotifier<WalletState> {
  final WalletRepository _repo;

  WalletNotifier(this._repo) : super(WalletState(isLoading: true)) {
    _initialize();
  }

  Future<void> _initialize() async {
    try {
      final isCreated = await _repo.isWalletCreated();
      if (isCreated) {
        await refreshAll();
      } else {
        state = WalletState();
      }
    } catch (e) {
      state = WalletState(error: e.toString());
    }
  }

  Future<void> refreshAll() async {
    state = WalletState(isLoading: true);
    try {
      final balance = await _repo.getBalance();
      final addresses = await _repo.getAddresses();
      final txs = await _repo.getTransactions();
      final accounts = addresses.isNotEmpty
          ? [
              WalletAccount(
                name: 'Main Account',
                index: 0,
                derivationPath: "m/84'/0'/0'",
                externalAddresses: addresses,
                internalAddresses: [],
                nextExternalIndex: addresses.length,
                nextInternalIndex: 0,
                isActive: true,
              )
            ]
          : <WalletAccount>[];

      state = WalletState(
        balance: balance,
        accounts: accounts,
        transactions: txs,
      );
    } catch (e) {
      state = WalletState(error: e.toString());
    }
  }

  Future<void> createWallet({required String name, required String passphrase, String? network}) async {
    state = WalletState(isLoading: true);
    try {
      await _repo.createWallet(
        name: name,
        passphrase: passphrase,
        network: network ?? 'mainnet',
      );
      await refreshAll();
    } catch (e) {
      state = WalletState(error: e.toString());
    }
  }

  Future<void> restoreWallet({required String seedPhrase, required String passphrase, String? network}) async {
    state = WalletState(isLoading: true);
    try {
      await _repo.restoreWallet(
        seedPhrase: seedPhrase,
        passphrase: passphrase,
        network: network ?? 'mainnet',
      );
      await refreshAll();
    } catch (e) {
      state = WalletState(error: e.toString());
    }
  }

  Future<void> sendTransaction({
    required String address,
    required int amountSats,
    required int feeRate,
  }) async {
    try {
      await _repo.sendTransaction(
        address: address,
        amountSats: amountSats,
        feeRate: feeRate,
      );
      await refreshAll();
    } catch (e) {
      state = WalletState(
        balance: state.balance,
        accounts: state.accounts,
        transactions: state.transactions,
        error: e.toString(),
      );
    }
  }

  void setLoading(bool loading) {
    state = WalletState(
      balance: state.balance,
      accounts: state.accounts,
      transactions: state.transactions,
      isLoading: loading,
    );
  }
}

final walletProvider = StateNotifierProvider<WalletNotifier, WalletState>((ref) {
  final repo = ref.watch(walletRepositoryProvider);
  return WalletNotifier(repo);
});

final isWalletCreatedProvider = FutureProvider<bool>((ref) async {
  final repo = ref.watch(walletRepositoryProvider);
  return repo.isWalletCreated();
});

final walletBalanceProvider = FutureProvider<WalletBalance>((ref) async {
  final repo = ref.watch(walletRepositoryProvider);
  return repo.getBalance();
});

final walletTransactionsProvider = FutureProvider.family<List<WalletTransaction>, ({int? limit, int? offset})>((ref, params) async {
  final repo = ref.watch(walletRepositoryProvider);
  return repo.getTransactions(limit: params.limit, offset: params.offset);
});

final walletAddressesProvider = FutureProvider<List<String>>((ref) async {
  final repo = ref.watch(walletRepositoryProvider);
  return repo.getAddresses();
});

final walletUtxosProvider = FutureProvider<List<WalletUtxo>>((ref) async {
  final repo = ref.watch(walletRepositoryProvider);
  return repo.getUtxos();
});

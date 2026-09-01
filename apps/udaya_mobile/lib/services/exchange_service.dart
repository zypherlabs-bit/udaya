import 'dart:math';
import 'package:udaya_mobile/domain/entities/exchange_entity.dart';
import 'package:udaya_mobile/core/errors/app_exceptions.dart';

class ExchangeService {
  static const _defaultRpcEndpoint = 'localhost:8332';
  static const _requiredConfirmations = 6;
  static const _minWithdrawalAmount = 0.001;
  static const _maxWithdrawalAmount = 1000000.0;

  String _rpcEndpoint = _defaultRpcEndpoint;
  bool _isTestnet = false;
  final List<DepositRecord> _deposits = [];
  final List<WithdrawalRequest> _withdrawals = [];
  final List<ExchangeRate> _rates = [];
  final Map<String, DepositAddress> _addresses = {};

  String get rpcEndpoint => _rpcEndpoint;
  bool get isTestnet => _isTestnet;
  List<DepositRecord> get deposits => List.unmodifiable(_deposits);
  List<WithdrawalRequest> get withdrawals => List.unmodifiable(_withdrawals);
  List<ExchangeRate> get rates => List.unmodifiable(_rates);
  Map<String, DepositAddress> get addresses => Map.unmodifiable(_addresses);

  void configureRpc(String endpoint, {bool isTestnet = false}) {
    _rpcEndpoint = endpoint;
    _isTestnet = isTestnet;
  }

  RpcConnectionStatus checkRpcConnection() {
    final errors = <String>[];

    if (_rpcEndpoint.isEmpty) {
      errors.add('RPC endpoint is not configured');
    }

    final random = Random();
    final latency = _rpcEndpoint.isNotEmpty ? 10 + random.nextInt(90) : -1;

    if (latency == -1 || errors.isNotEmpty) {
      return RpcConnectionStatus(
        isConnected: false,
        endpoint: _rpcEndpoint.isEmpty ? null : _rpcEndpoint,
        latencyMs: -1,
        blockHeight: 0,
        lastChecked: DateTime.now(),
        errors: errors,
      );
    }

    return RpcConnectionStatus(
      isConnected: true,
      endpoint: _rpcEndpoint,
      latencyMs: latency,
      blockHeight: 800000 + random.nextInt(10000),
      lastChecked: DateTime.now(),
      errors: const [],
    );
  }

  Future<DepositAddress> generateDepositAddress(String label) async {
    final address = _isTestnet
        ? 'tb1q${_randomHex(38)}'
        : 'bc1q${_randomHex(38)}';

    final depositAddress = DepositAddress(
      address: address,
      label: label,
      createdAt: DateTime.now(),
      isUsed: false,
      confirmations: 0,
    );

    _addresses[address] = depositAddress;
    return depositAddress;
  }

  Future<DepositRecord> checkDepositStatus(String txid, String address, double amount) async {
    final existing = _deposits.where((d) => d.txid == txid).toList();
    if (existing.isNotEmpty) return existing.first;

    final confirmations = Random().nextInt(_requiredConfirmations);
    DepositStatus status;
    if (confirmations == 0) {
      status = DepositStatus.pending;
    } else if (confirmations < _requiredConfirmations) {
      status = DepositStatus.confirming;
    } else {
      status = DepositStatus.confirmed;
    }

    final record = DepositRecord(
      id: 'dep_${DateTime.now().millisecondsSinceEpoch}',
      txid: txid,
      address: address,
      amount: amount,
      confirmations: confirmations,
      timestamp: DateTime.now(),
      status: status,
    );

    _deposits.add(record);
    return record;
  }

  Future<List<DepositRecord>> getDepositHistory() async {
    return List.unmodifiable(_deposits);
  }

  Future<List<DepositAddress>> getDepositAddresses() async {
    return List.unmodifiable(_addresses.values);
  }

  Future<WithdrawalRequest> createWithdrawal({
    required String toAddress,
    required double amount,
    required double fee,
  }) async {
    if (amount < _minWithdrawalAmount) {
      throw ValidationException(
        message: 'Withdrawal amount must be at least $_minWithdrawalAmount UDYA',
        code: 'WITHDRAWAL_BELOW_MIN',
      );
    }

    if (amount > _maxWithdrawalAmount) {
      throw ValidationException(
        message: 'Withdrawal amount exceeds maximum of $_maxWithdrawalAmount UDYA',
        code: 'WITHDRAWAL_ABOVE_MAX',
      );
    }

    final request = WithdrawalRequest(
      id: 'wit_${DateTime.now().millisecondsSinceEpoch}',
      toAddress: toAddress,
      amount: amount,
      fee: fee,
      status: WithdrawalStatus.pending,
      createdAt: DateTime.now(),
    );

    _withdrawals.add(request);
    return request;
  }

  Future<WithdrawalRequest> processWithdrawal(String withdrawalId) async {
    final index = _withdrawals.indexWhere((w) => w.id == withdrawalId);
    if (index == -1) {
      throw WalletException(
        message: 'Withdrawal not found',
        code: 'WITHDRAWAL_NOT_FOUND',
      );
    }

    final request = _withdrawals[index];
    if (request.status != WithdrawalStatus.pending) {
      return request;
    }

    final updated = WithdrawalRequest(
      id: request.id,
      toAddress: request.toAddress,
      amount: request.amount,
      fee: request.fee,
      status: WithdrawalStatus.processing,
      createdAt: request.createdAt,
      txid: 'tx_${_randomHex(64)}',
      confirmations: 0,
    );

    _withdrawals[index] = updated;
    return updated;
  }

  Future<WithdrawalRequest> confirmWithdrawal(String withdrawalId, int confirmations) async {
    final index = _withdrawals.indexWhere((w) => w.id == withdrawalId);
    if (index == -1) {
      throw WalletException(
        message: 'Withdrawal not found',
        code: 'WITHDRAWAL_NOT_FOUND',
      );
    }

    final request = _withdrawals[index];
    if (request.status != WithdrawalStatus.processing) {
      return request;
    }

    final status = confirmations >= 3
        ? WithdrawalStatus.completed
        : WithdrawalStatus.processing;

    final updated = WithdrawalRequest(
      id: request.id,
      toAddress: request.toAddress,
      amount: request.amount,
      fee: request.fee,
      status: status,
      createdAt: request.createdAt,
      txid: request.txid,
      confirmations: confirmations,
    );

    _withdrawals[index] = updated;
    return updated;
  }

  Future<void> cancelWithdrawal(String withdrawalId) async {
    final index = _withdrawals.indexWhere((w) => w.id == withdrawalId);
    if (index == -1) {
      throw WalletException(
        message: 'Withdrawal not found',
        code: 'WITHDRAWAL_NOT_FOUND',
      );
    }

    final request = _withdrawals[index];
    if (request.status != WithdrawalStatus.pending) {
      throw WalletException(
        message: 'Cannot cancel withdrawal in status: ${request.status}',
        code: 'WITHDRAWAL_NOT_CANCELLABLE',
      );
    }

    _withdrawals[index] = WithdrawalRequest(
      id: request.id,
      toAddress: request.toAddress,
      amount: request.amount,
      fee: request.fee,
      status: WithdrawalStatus.cancelled,
      createdAt: request.createdAt,
    );
  }

  Future<List<WithdrawalRequest>> getWithdrawalHistory() async {
    return List.unmodifiable(_withdrawals);
  }

  Future<ExchangeRate> getExchangeRate(String pair) async {
    final existing = _rates.where((r) => r.pair == pair).toList();
    if (existing.isNotEmpty) {
      final latest = existing.last;
      final age = DateTime.now().difference(latest.updatedAt);
      if (age.inMinutes < 5) return latest;
    }

    final random = Random();
    final basePrice = pair.contains('USD') ? 1.0 : pair.contains('ETH') ? 0.001 : 0.001;
    final price = basePrice * (0.9 + random.nextDouble() * 0.2);
    final change = (random.nextDouble() - 0.5) * 10;

    final rate = ExchangeRate(
      pair: pair,
      price: price,
      change24h: change,
      updatedAt: DateTime.now(),
    );

    _rates.add(rate);
    return rate;
  }

  Future<List<ExchangeRate>> getSupportedRates() async {
    final pairs = ['UDYA/USD', 'UDYA/ETH'];
    final rates = <ExchangeRate>[];
    for (final pair in pairs) {
      rates.add(await getExchangeRate(pair));
    }
    return rates;
  }

  DepositWithdrawalStats getStats() {
    final totalDeposited = _deposits.fold<double>(
      0,
      (sum, d) => d.status == DepositStatus.confirmed ? sum + d.amount : sum,
    );
    final totalWithdrawn = _withdrawals.fold<double>(
      0,
      (sum, w) => w.status == WithdrawalStatus.completed ? sum + w.amount : sum,
    );

    return DepositWithdrawalStats(
      totalDeposits: _deposits.length,
      totalWithdrawals: _withdrawals.length,
      totalDeposited: totalDeposited,
      totalWithdrawn: totalWithdrawn,
      pendingDeposits: _deposits.where((d) => d.status == DepositStatus.pending || d.status == DepositStatus.confirming).length,
      pendingWithdrawals: _withdrawals.where((w) => w.status == WithdrawalStatus.pending || w.status == WithdrawalStatus.processing).length,
    );
  }

  String _randomHex(int length) {
    final random = Random();
    final chars = '0123456789abcdef';
    return String.fromCharCodes(Iterable.generate(length, (_) => chars.codeUnitAt(random.nextInt(chars.length))));
  }
}

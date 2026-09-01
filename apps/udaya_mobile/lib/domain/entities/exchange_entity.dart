class DepositAddress {
  final String address;
  final String label;
  final DateTime createdAt;
  final bool isUsed;
  final int confirmations;

  DepositAddress({
    required this.address,
    required this.label,
    required this.createdAt,
    required this.isUsed,
    required this.confirmations,
  });
}

class DepositRecord {
  final String id;
  final String txid;
  final String address;
  final double amount;
  final int confirmations;
  final DateTime timestamp;
  final DepositStatus status;

  DepositRecord({
    required this.id,
    required this.txid,
    required this.address,
    required this.amount,
    required this.confirmations,
    required this.timestamp,
    required this.status,
  });
}

enum DepositStatus { pending, confirming, confirmed, failed }

class WithdrawalRequest {
  final String id;
  final String toAddress;
  final double amount;
  final double fee;
  final WithdrawalStatus status;
  final DateTime createdAt;
  final String? txid;
  final int confirmations;
  final String? errorMessage;

  WithdrawalRequest({
    required this.id,
    required this.toAddress,
    required this.amount,
    required this.fee,
    required this.status,
    required this.createdAt,
    this.txid,
    this.confirmations = 0,
    this.errorMessage,
  });
}

enum WithdrawalStatus { pending, processing, completed, failed, cancelled }

class ExchangeRate {
  final String pair;
  final double price;
  final double change24h;
  final DateTime updatedAt;

  ExchangeRate({
    required this.pair,
    required this.price,
    required this.change24h,
    required this.updatedAt,
  });
}

class RpcConnectionStatus {
  final bool isConnected;
  final String? endpoint;
  final int latencyMs;
  final int blockHeight;
  final DateTime lastChecked;
  final List<String> errors;

  RpcConnectionStatus({
    required this.isConnected,
    this.endpoint,
    required this.latencyMs,
    required this.blockHeight,
    required this.lastChecked,
    this.errors = const [],
  });
}

class DepositWithdrawalStats {
  final int totalDeposits;
  final int totalWithdrawals;
  final double totalDeposited;
  final double totalWithdrawn;
  final int pendingDeposits;
  final int pendingWithdrawals;

  DepositWithdrawalStats({
    required this.totalDeposits,
    required this.totalWithdrawals,
    required this.totalDeposited,
    required this.totalWithdrawn,
    required this.pendingDeposits,
    required this.pendingWithdrawals,
  });
}

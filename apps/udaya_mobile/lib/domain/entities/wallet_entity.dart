import 'package:udaya_mobile/core/constants/app_constants.dart';

class WalletAccount {
  final String name;
  final int index;
  final String derivationPath; // BIP44, BIP49, BIP84, BIP86
  final List<String> externalAddresses;
  final List<String> internalAddresses;
  final int nextExternalIndex;
  final int nextInternalIndex;
  final bool isActive;

  WalletAccount({
    required this.name,
    required this.index,
    required this.derivationPath,
    required this.externalAddresses,
    required this.internalAddresses,
    required this.nextExternalIndex,
    required this.nextInternalIndex,
    this.isActive = false,
  });

  WalletAccount copyWith({
    String? name,
    int? index,
    String? derivationPath,
    List<String>? externalAddresses,
    List<String>? internalAddresses,
    int? nextExternalIndex,
    int? nextInternalIndex,
    bool? isActive,
  }) {
    return WalletAccount(
      name: name ?? this.name,
      index: index ?? this.index,
      derivationPath: derivationPath ?? this.derivationPath,
      externalAddresses: externalAddresses ?? this.externalAddresses,
      internalAddresses: internalAddresses ?? this.internalAddresses,
      nextExternalIndex: nextExternalIndex ?? this.nextExternalIndex,
      nextInternalIndex: nextInternalIndex ?? this.nextInternalIndex,
      isActive: isActive ?? this.isActive,
    );
  }
}

class WalletBalance {
  final int total;
  final int spendable;
  final int immature;
  final int unconfirmed;

  WalletBalance({
    required this.total,
    required this.spendable,
    required this.immature,
    required this.unconfirmed,
  });

  double get totalUdya => total / AppConstants.satoshisPerCoin;
  double get spendableUdya => spendable / AppConstants.satoshisPerCoin;
}

class WalletTransaction {
  final String txid;
  final int timestamp;
  final TransactionDirection direction;
  final TransactionStatus status;
  final int amount;
  final int fee;
  final int confirmations;
  final int? blockHeight;
  final String? address;
  final String? label;

  WalletTransaction({
    required this.txid,
    required this.timestamp,
    required this.direction,
    required this.status,
    required this.amount,
    required this.fee,
    required this.confirmations,
    this.blockHeight,
    this.address,
    this.label,
  });

  double get amountUdya => amount / AppConstants.satoshisPerCoin;
}

enum TransactionDirection { sent, received, selfTransfer }
enum TransactionStatus { pending, confirmed, failed }

class WalletUtxo {
  final String txid;
  final int vout;
  final int value;
  final String address;
  final String scriptPubkey;
  final int height;
  final int confirmations;
  final bool isCoinbase;
  final bool isSpent;
  final bool isFrozen;

  WalletUtxo({
    required this.txid,
    required this.vout,
    required this.value,
    required this.address,
    required this.scriptPubkey,
    required this.height,
    required this.confirmations,
    required this.isCoinbase,
    required this.isSpent,
    required this.isFrozen,
  });

  double get valueUdya => value / AppConstants.satoshisPerCoin;
}

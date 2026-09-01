import 'package:udaya_mobile/core/constants/app_constants.dart';

class PeerInfo {
  final String id;
  final String address;
  final String userAgent;
  final int height;
  final int pingTimeMs;
  final bool isOutbound;
  final bool isConnected;
  final DateTime connectedSince;

  PeerInfo({
    required this.id,
    required this.address,
    required this.userAgent,
    required this.height,
    required this.pingTimeMs,
    required this.isOutbound,
    required this.isConnected,
    required this.connectedSince,
  });
}

class BlockInfo {
  final String hash;
  final int height;
  final int version;
  final String previousBlockHash;
  final String merkleRoot;
  final int timestamp;
  final int bits;
  final int nonce;
  final int transactionCount;
  final int size;
  final int weight;
  final int difficulty;
  final String chainwork;

  BlockInfo({
    required this.hash,
    required this.height,
    required this.version,
    required this.previousBlockHash,
    required this.merkleRoot,
    required this.timestamp,
    required this.bits,
    required this.nonce,
    required this.transactionCount,
    required this.size,
    required this.weight,
    required this.difficulty,
    required this.chainwork,
  });
}

class TransactionInfo {
  final String txid;
  final String wtxid;
  final int version;
  final int sizeBytes;
  final int vsize;
  final int weight;
  final int fee;
  final int feeRate;
  final int lockTime;
  final int? blockHeight;
  final String? blockHash;
  final int? blockTime;
  final int confirmations;
  final bool isCoinbase;
  final int timestamp;
  final List<TxInputInfo> inputs;
  final List<TxOutputInfo> outputs;

  TransactionInfo({
    required this.txid,
    required this.wtxid,
    required this.version,
    required this.sizeBytes,
    required this.vsize,
    required this.weight,
    required this.fee,
    required this.feeRate,
    required this.lockTime,
    this.blockHeight,
    this.blockHash,
    this.blockTime,
    required this.confirmations,
    required this.isCoinbase,
    required this.timestamp,
    required this.inputs,
    required this.outputs,
  });
}

class TxInputInfo {
  final String txid;
  final int vout;
  final String? address;
  final int? value;
  final String? scriptSigHex;

  TxInputInfo({
    required this.txid,
    required this.vout,
    this.address,
    this.value,
    this.scriptSigHex,
  });
}

class TxOutputInfo {
  final int value;
  final String address;
  final String scriptPubkeyHex;
  final bool isSpent;

  TxOutputInfo({
    required this.value,
    required this.address,
    required this.scriptPubkeyHex,
    required this.isSpent,
  });

  double get valueUdya => value / AppConstants.satoshisPerCoin;
}

class MiningInfo {
  final bool isMining;
  final double hashrate;
  final int blocksFound;
  final double difficulty;
  final int blockReward;
  final int networkHeight;
  final String miningAddress;
  final int minerThreads;

  MiningInfo({
    required this.isMining,
    required this.hashrate,
    required this.blocksFound,
    required this.difficulty,
    required this.blockReward,
    required this.networkHeight,
    required this.miningAddress,
    required this.minerThreads,
  });
}

class MempoolInfo {
  final int size;
  final int bytes;
  final int maxMempool;
  final double mempoolMinFee;
  final double minRelayTxFee;

  MempoolInfo({
    required this.size,
    required this.bytes,
    required this.maxMempool,
    required this.mempoolMinFee,
    required this.minRelayTxFee,
  });
}

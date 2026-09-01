import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/exchange_entity.dart';
import 'package:udaya_mobile/services/exchange_service.dart';

final exchangeServiceProvider = Provider<ExchangeService>((ref) {
  return ExchangeService();
});

final rpcConnectionStatusProvider = FutureProvider<RpcConnectionStatus>((ref) async {
  final service = ref.watch(exchangeServiceProvider);
  return service.checkRpcConnection();
});

final depositAddressesProvider = FutureProvider<List<DepositAddress>>((ref) async {
  final service = ref.watch(exchangeServiceProvider);
  return service.getDepositAddresses();
});

final depositHistoryProvider = FutureProvider<List<DepositRecord>>((ref) async {
  final service = ref.watch(exchangeServiceProvider);
  return service.getDepositHistory();
});

final withdrawalHistoryProvider = FutureProvider<List<WithdrawalRequest>>((ref) async {
  final service = ref.watch(exchangeServiceProvider);
  return service.getWithdrawalHistory();
});

final exchangeRatesProvider = FutureProvider<List<ExchangeRate>>((ref) async {
  final service = ref.watch(exchangeServiceProvider);
  return service.getSupportedRates();
});

final depositWithdrawalStatsProvider = Provider<DepositWithdrawalStats>((ref) {
  return ref.watch(exchangeServiceProvider).getStats();
});

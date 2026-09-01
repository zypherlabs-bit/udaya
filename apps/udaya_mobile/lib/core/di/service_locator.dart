import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:udaya_mobile/data/datasources/local/shared_prefs_source.dart';
import 'package:udaya_mobile/data/datasources/ffi/udaya_ffi.dart';
import 'package:udaya_mobile/data/datasources/remote/udaya_rpc_client.dart';
import 'package:udaya_mobile/data/repositories/node_repository_impl.dart';
import 'package:udaya_mobile/data/repositories/mining_repository_impl.dart';
import 'package:udaya_mobile/data/repositories/blockchain_repository_impl.dart';
import 'package:udaya_mobile/data/repositories/wallet_repository_impl.dart';
import 'package:udaya_mobile/data/repositories/wallet_remote_repository.dart';
import 'package:udaya_mobile/services/node_service.dart';
import 'package:udaya_mobile/services/monitoring_service.dart';
import 'package:udaya_mobile/services/secure_storage_service.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/providers/monitoring_provider.dart';
import 'package:udaya_mobile/presentation/providers/node_provider.dart';
import 'package:udaya_mobile/presentation/providers/wallet_provider.dart';
import 'package:udaya_mobile/presentation/routes/app_router.dart';
import 'package:flutter/material.dart';

ProviderScope createProviderScope({
  required Widget child,
  required SharedPreferences prefs,
}) {
  final ffi = UdayaFfi();
  final prefsSource = SharedPrefsSource(prefs);
  final secureStorage = SecureStorageService();
  final nodeService = NodeService();
  final monitoringService = MonitoringService(nodeService, secureStorage);

  final isRemoteWallet = prefs.getBool('wallet_remote_mode') ?? false;
  final nodeRepo = NodeRepositoryImpl(prefsSource);
  final walletRepo = isRemoteWallet
      ? WalletRemoteRepository(UdayaRpcClient(
          host: prefsSource.getString('rpc_host') ?? 'localhost',
          port: int.tryParse(prefsSource.getString('rpc_port') ?? '8332') ?? 8332,
          username: 'Udaya',
          password: '',
          useTls: false,
        ))
      : WalletRepositoryImpl(ffi, FlutterSecureStorage());

  return ProviderScope(
    overrides: [
      sharedPrefsSourceProvider.overrideWithValue(prefsSource),
      nodeServiceProvider.overrideWithValue(nodeService),
      monitoringServiceProvider.overrideWithValue(monitoringService),
      nodeRepositoryProvider.overrideWithValue(nodeRepo),
      miningRepositoryProvider.overrideWithValue(MiningRepositoryImpl(null, nodeRepo)),
      blockchainRepositoryProvider.overrideWithValue(BlockchainRepositoryImpl(null)),
      walletRepositoryProvider.overrideWithValue(walletRepo),
    ],
    child: child,
  );
}
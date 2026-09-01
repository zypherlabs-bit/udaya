import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'package:udaya_mobile/domain/repositories/wallet_repository.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/screens/wallet_screen.dart';
import 'package:udaya_mobile/presentation/screens/send_screen.dart';
import 'package:udaya_mobile/presentation/screens/receive_screen.dart';
import 'package:udaya_mobile/presentation/screens/transactions_screen.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';

class MockWalletRepository implements WalletRepository {
  bool _walletCreated = false;
  final List<WalletTransaction> _transactions = [];
  final int _balance = 0;

  @override
  Future<String> createWallet({required String name, required String passphrase, String? network}) async {
    _walletCreated = true;
    return 'test seed phrase';
  }

  @override
  Future<String> restoreWallet({required String seedPhrase, required String passphrase, String? network}) async {
    _walletCreated = true;
    return seedPhrase;
  }

  @override
  Future<String> generateAddress({int? accountIndex}) async => 'UDAYA_test123456789';

  @override
  Future<List<String>> getAddresses({int? accountIndex}) async => ['UDAYA_test123456789'];

  @override
  Future<WalletBalance> getBalance() async => WalletBalance(total: _balance, spendable: _balance, immature: 0, unconfirmed: 0);

  @override
  Future<List<WalletTransaction>> getTransactions({int? limit, int? offset}) async => _transactions;

  @override
  Future<List<WalletUtxo>> getUtxos() async => [];

  @override
  Future<String> sendTransaction({required String address, required int amountSats, required int feeRate}) async => 'mock_txid_12345';

  @override
  Future<String> signMessage({required String message, required String address}) async => 'mock_signature';

  @override
  Future<bool> verifyMessage({required String message, required String signature, required String address}) async => true;

  @override
  Future<bool> isWalletCreated() async => _walletCreated;

  @override
  Future<String> exportSeedPhrase({String? passphrase}) async => 'test seed phrase';

  @override
  Future<bool> lockWallet() async => true;

  @override
  Future<bool> unlockWallet({required String passphrase}) async => true;

  @override
  Future<bool> isLocked() async => false;

  @override
  Future<void> backupWallet({required String destinationPath}) async {}
}

void main() {
  group('WalletScreen Widget Tests', () {
    late MockWalletRepository mockRepo;

    setUp(() {
      mockRepo = MockWalletRepository();
    });

    testWidgets('should render without crashing when no wallet exists', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            walletRepositoryProvider.overrideWithValue(mockRepo),
            walletProvider.overrideWith((ref) => WalletNotifier(mockRepo)),
          ],
          child: MaterialApp(
            theme: UdayaTheme.lightTheme,
            home: const WalletScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      expect(find.byType(WalletScreen), findsOneWidget);
    });

    testWidgets('should render balance card when wallet exists', (WidgetTester tester) async {
      await mockRepo.createWallet(name: 'Test', passphrase: 'test');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            walletRepositoryProvider.overrideWithValue(mockRepo),
            walletProvider.overrideWith((ref) => WalletNotifier(mockRepo)),
          ],
          child: MaterialApp(
            theme: UdayaTheme.lightTheme,
            home: const WalletScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      expect(find.byType(GlassCard), findsWidgets);
    });
  });

  group('SendScreen Widget Tests', () {
    late MockWalletRepository mockRepo;

    setUp(() {
      mockRepo = MockWalletRepository();
    });

    testWidgets('should render send form when wallet exists', (WidgetTester tester) async {
      await mockRepo.createWallet(name: 'Test', passphrase: 'test');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            walletRepositoryProvider.overrideWithValue(mockRepo),
            walletProvider.overrideWith((ref) => WalletNotifier(mockRepo)),
          ],
          child: MaterialApp(
            theme: UdayaTheme.lightTheme,
            home: const SendScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      expect(find.byType(SendScreen), findsOneWidget);
      expect(find.byType(Form), findsOneWidget);
    });
  });

  group('ReceiveScreen Widget Tests', () {
    late MockWalletRepository mockRepo;

    setUp(() {
      mockRepo = MockWalletRepository();
    });

    testWidgets('should render receive screen when wallet exists', (WidgetTester tester) async {
      await mockRepo.createWallet(name: 'Test', passphrase: 'test');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            walletRepositoryProvider.overrideWithValue(mockRepo),
            walletProvider.overrideWith((ref) => WalletNotifier(mockRepo)),
          ],
          child: MaterialApp(
            theme: UdayaTheme.lightTheme,
            home: const ReceiveScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      expect(find.byType(ReceiveScreen), findsOneWidget);
    });
  });

  group('TransactionsScreen Widget Tests', () {
    late MockWalletRepository mockRepo;

    setUp(() {
      mockRepo = MockWalletRepository();
    });

    testWidgets('should render transactions screen when wallet exists', (WidgetTester tester) async {
      await mockRepo.createWallet(name: 'Test', passphrase: 'test');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            walletRepositoryProvider.overrideWithValue(mockRepo),
            walletProvider.overrideWith((ref) => WalletNotifier(mockRepo)),
          ],
          child: MaterialApp(
            theme: UdayaTheme.lightTheme,
            home: const TransactionsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      expect(find.byType(TransactionsScreen), findsOneWidget);
    });
  });
}

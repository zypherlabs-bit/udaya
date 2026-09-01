import 'package:flutter_test/flutter_test.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';
import 'package:udaya_mobile/core/errors/app_exceptions.dart';

void main() {
  group('AppConstants', () {
    test('should have correct satoshis per coin', () {
      expect(AppConstants.satoshisPerCoin, 100000000);
    });

    test('should have correct default port', () {
      expect(AppConstants.defaultPort, 9798);
    });

    test('should have correct testnet port', () {
      expect(AppConstants.defaultTestnetPort, 19798);
    });

    test('should have correct block target time', () {
      expect(AppConstants.blockTargetTimeSecs, 600);
    });
  });

  group('AppException', () {
    test('should create with message and code', () {
      final exception = AppException(message: 'Test error', code: 'TEST_CODE');
      expect(exception.message, 'Test error');
      expect(exception.code, 'TEST_CODE');
    });

    test('should create FfiException', () {
      final exception = FfiException(message: 'FFI error');
      expect(exception.message, 'FFI error');
    });

    test('should create NodeException', () {
      final exception = NodeException(message: 'Node error');
      expect(exception.message, 'Node error');
    });

    test('should create WalletException', () {
      final exception = WalletException(message: 'Wallet error');
      expect(exception.message, 'Wallet error');
    });
  });
}

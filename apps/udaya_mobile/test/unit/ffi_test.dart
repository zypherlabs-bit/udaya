import 'dart:ffi';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:udaya_mobile/data/datasources/ffi/udaya_ffi.dart';

bool get _isLibraryAvailable {
  try {
    if (Platform.isAndroid || Platform.isLinux) {
      DynamicLibrary.open('libudaya_ffi_mobile.so');
      return true;
    } else if (Platform.isWindows) {
      DynamicLibrary.open('udaya_ffi_mobile.dll');
      return true;
    } else if (Platform.isMacOS) {
      DynamicLibrary.open('libudaya_ffi_mobile.dylib');
      return true;
    } else if (Platform.isIOS) {
      DynamicLibrary.open('udaya_ffi_mobile.framework/udaya_ffi_mobile');
      return true;
    }
    return false;
  } catch (_) {
    return false;
  }
}

void main() {
  if (!_isLibraryAvailable) {
    test('FFI library not available - skipping tests', () {
      markTestSkipped('Native library not built for this platform');
    });
    return;
  }

  group('UdayaFfi', () {
    late UdayaFfi ffi;

    setUp(() {
      ffi = UdayaFfi();
    });

    test('walletCreate with mainnet returns non-null handle', () {
      final handle = ffi.walletCreate('Test Wallet', 'mainnet');
      expect(handle.address, isNonZero);
      ffi.walletDestroy(handle);
    });

    test('walletCreateNew returns seed phrase', () {
      final handle = ffi.walletCreate('Test Wallet', 'mainnet');
      expect(handle.address, isNonZero);

      final seed = ffi.walletCreateNew(handle);
      expect(seed, isNotEmpty);
      expect(seed.split(' ').length, greaterThanOrEqualTo(12));

      ffi.walletDestroy(handle);
    });

    test('walletGenerateAddress returns address', () {
      final handle = ffi.walletCreate('Test Wallet', 'mainnet');
      expect(handle.address, isNonZero);

      final address = ffi.walletGenerateAddress(handle);
      expect(address, isNotEmpty);

      ffi.walletDestroy(handle);
    });

    test('walletGetBalance returns non-negative value', () {
      final handle = ffi.walletCreate('Test Wallet', 'mainnet');
      expect(handle.address, isNonZero);

      final balance = ffi.walletGetBalance(handle);
      expect(balance, greaterThanOrEqualTo(0));

      ffi.walletDestroy(handle);
    });

    test('walletGetTransactions returns list', () {
      final handle = ffi.walletCreate('Test Wallet', 'mainnet');
      expect(handle.address, isNonZero);

      final txs = ffi.walletGetTransactions(handle);
      expect(txs, isA<List>());

      ffi.walletDestroy(handle);
    });

    test('diagnosticCheck returns success', () {
      final result = ffi.diagnosticCheck();
      expect(result, equals(0));
    });
  });
}

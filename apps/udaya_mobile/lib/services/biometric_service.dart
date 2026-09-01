import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:local_auth/local_auth.dart';

class BiometricService {
  static final LocalAuthentication _auth = LocalAuthentication();
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage(
    aOptions: AndroidOptions(encryptedSharedPreferences: true),
  );

  static const String _biometricEnabledKey = 'biometric_enabled';
  static const String _appLockEnabledKey = 'app_lock_enabled';
  static const String _lastAuthTimeKey = 'last_auth_time';
  static const String _appLockTimeoutKey = 'app_lock_timeout_minutes';

  Future<bool> isBiometricAvailable() async {
    try {
      final canCheck = await _auth.canCheckBiometrics;
      final biometrics = await _auth.getAvailableBiometrics();
      return canCheck && biometrics.isNotEmpty;
    } on PlatformException {
      return false;
    } catch (e) {
      return false;
    }
  }

  Future<List<BiometricType>> getAvailableBiometrics() async {
    try {
      return await _auth.getAvailableBiometrics();
    } on PlatformException {
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<bool> hasStrongBiometric() async {
    try {
      final biometrics = await _auth.getAvailableBiometrics();
      return biometrics.contains(BiometricType.strong) ||
          biometrics.contains(BiometricType.fingerprint) ||
          biometrics.contains(BiometricType.face);
    } on PlatformException {
      return false;
    } catch (e) {
      return false;
    }
  }

  Future<bool> authenticate({
    String? reason,
    bool useErrorDialogs = true,
    bool stickyAuth = true,
  }) async {
    try {
      return await _auth.authenticate(
        localizedReason: reason ?? 'Authenticate to access Udaya',
        options: AuthenticationOptions(
          useErrorDialogs: useErrorDialogs,
          stickyAuth: stickyAuth,
          biometricOnly: false,
        ),
      );
    } on PlatformException catch (e) {
      if (e.code == 'NotAvailable') {
        return false;
      }
      if (e.code == 'NotEnrolled') {
        return false;
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  Future<void> stopAuthentication() async {
    try {
      await _auth.stopAuthentication();
    } on PlatformException {
      // Ignore
    } catch (e) {
      // Ignore
    }
  }

  Future<bool> isBiometricEnabled() async {
    try {
      final value = await _secureStorage.read(key: _biometricEnabledKey);
      return value == 'true';
    } catch (e) {
      return false;
    }
  }

  Future<void> setBiometricEnabled(bool enabled) async {
    try {
      await _secureStorage.write(key: _biometricEnabledKey, value: enabled.toString());
    } catch (e) {
      // Ignore
    }
  }

  Future<bool> isAppLockEnabled() async {
    try {
      final value = await _secureStorage.read(key: _appLockEnabledKey);
      return value == 'true';
    } catch (e) {
      return false;
    }
  }

  Future<void> setAppLockEnabled(bool enabled) async {
    try {
      await _secureStorage.write(key: _appLockEnabledKey, value: enabled.toString());
    } catch (e) {
      // Ignore
    }
  }

  Future<DateTime?> getLastAuthTime() async {
    try {
      final value = await _secureStorage.read(key: _lastAuthTimeKey);
      if (value == null) return null;
      return DateTime.tryParse(value);
    } catch (e) {
      return null;
    }
  }

  Future<void> setLastAuthTime(DateTime time) async {
    try {
      await _secureStorage.write(key: _lastAuthTimeKey, value: time.toIso8601String());
    } catch (e) {
      // Ignore
    }
  }

  Future<void> clearAuthData() async {
    try {
      await _secureStorage.delete(key: _lastAuthTimeKey);
    } catch (e) {
      // Ignore
    }
  }

  Future<int?> getAppLockTimeoutMinutes() async {
    try {
      final value = await _secureStorage.read(key: _appLockTimeoutKey);
      if (value == null) return 5;
      if (value == 'never') return null;
      return int.tryParse(value) ?? 5;
    } catch (e) {
      return 5;
    }
  }

  Future<void> setAppLockTimeoutMinutes(int? minutes) async {
    try {
      await _secureStorage.write(
        key: _appLockTimeoutKey,
        value: minutes == null ? 'never' : minutes.toString(),
      );
    } catch (e) {
      // Ignore
    }
  }
}

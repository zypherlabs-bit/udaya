import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Wrapper around flutter_secure_storage for secure key-value storage.
///
/// This service ensures sensitive data like seed phrases and encryption keys
/// are never exposed in logs or plain-text storage.
class SecureStorageService {
  static const _storage = FlutterSecureStorage(
    aOptions: AndroidOptions(
      encryptedSharedPreferences: true,
    ),
  );

  static const String _walletSeedKey = 'wallet_seed';
  static const String _walletEncryptionKeyKey = 'wallet_encryption_key';

  /// Writes a value to secure storage.
  Future<void> writeSecure(String key, String value) async {
    if (key == _walletSeedKey || key == _walletEncryptionKeyKey) {
      // Never log the value for sensitive keys
    }
    await _storage.write(key: key, value: value);
  }

  /// Reads a value from secure storage.
  Future<String?> readSecure(String key) async {
    final value = await _storage.read(key: key);
    return value;
  }

  /// Deletes a value from secure storage.
  Future<void> deleteSecure(String key) async {
    await _storage.delete(key: key);
  }

  /// Checks if a key exists in secure storage.
  Future<bool> containsSecure(String key) async {
    final value = await _storage.read(key: key);
    return value != null;
  }

  /// Writes binary data as a base64 string.
  Future<void> writeSecureBytes(String key, Uint8List bytes) async {
    await writeSecure(key, bytesToBase64(bytes));
  }

  /// Reads binary data from a base64 string.
  Future<Uint8List?> readSecureBytes(String key) async {
    final value = await readSecure(key);
    if (value == null) return null;
    return base64ToBytes(value);
  }

  /// Clears all secure storage.
  Future<void> clearAll() async {
    await _storage.deleteAll();
  }

  /// Returns all keys stored in secure storage.
  Future<List<String>> getAllKeys() async {
    return await _storage.readAll().then((map) => map.keys.toList());
  }

  String bytesToBase64(Uint8List bytes) {
    return base64.encode(bytes);
  }

  Uint8List base64ToBytes(String base64String) {
    return base64.decode(base64String);
  }
}

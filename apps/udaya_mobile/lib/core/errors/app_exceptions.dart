class AppException implements Exception {
  final String message;
  final String? code;
  final dynamic originalError;

  const AppException({
    required this.message,
    this.code,
    this.originalError,
  });

  @override
  String toString() => 'AppException($code): $message';
}

class FfiException extends AppException {
  const FfiException({required super.message, super.code, super.originalError});
}

class NodeException extends AppException {
  const NodeException({required super.message, super.code, super.originalError});
}

class WalletException extends AppException {
  const WalletException({required super.message, super.code, super.originalError});
}

class NetworkException extends AppException {
  const NetworkException({required super.message, super.code, super.originalError});
}

class StorageException extends AppException {
  const StorageException({required super.message, super.code, super.originalError});
}

class ValidationException extends AppException {
  const ValidationException({required super.message, super.code, super.originalError});
}

class BlockchainException extends AppException {
  const BlockchainException({required super.message, super.code, super.originalError});
}

class SecurityException extends AppException {
  const SecurityException({required super.message, super.code, super.originalError});
}

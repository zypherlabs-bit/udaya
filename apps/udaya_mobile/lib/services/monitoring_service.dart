import 'dart:async';
import 'package:udaya_mobile/services/node_service.dart';
import 'package:udaya_mobile/services/secure_storage_service.dart';

enum HealthStatus { healthy, degraded, unhealthy, unknown }

class HealthCheckResult {
  final HealthStatus status;
  final String message;
  final DateTime checkedAt;
  final Map<String, dynamic> details;

  HealthCheckResult({
    required this.status,
    required this.message,
    required this.checkedAt,
    this.details = const {},
  });
}

class MonitoringService {
  final NodeService nodeService;
  final SecureStorageService storage;

  StreamController<HealthCheckResult>? _healthController;
  Timer? _healthCheckTimer;
  Timer? _autoRestartTimer;

  int _consecutiveFailures = 0;
  static const int _maxConsecutiveFailures = 3;
  static const Duration _healthCheckInterval = Duration(seconds: 10);
  static const Duration _autoRestartDelay = Duration(seconds: 5);

  bool _isRunning = false;

  MonitoringService(this.nodeService, this.storage);

  Future<void> startMonitoring() async {
    if (_isRunning) return;
    _isRunning = true;
    _healthController ??= StreamController<HealthCheckResult>.broadcast();
    _runHealthCheck();
    _healthCheckTimer = Timer.periodic(_healthCheckInterval, (_) => _runHealthCheck());
  }

  Future<void> stopMonitoring() async {
    _healthCheckTimer?.cancel();
    _autoRestartTimer?.cancel();
    await _healthController?.close();
    _healthController = null;
    _isRunning = false;
  }

  Stream<HealthCheckResult> get healthStream {
    _healthController ??= StreamController<HealthCheckResult>.broadcast();
    return _healthController!.stream;
  }

  Future<void> _runHealthCheck() async {
    try {
      final status = await nodeService.getStatus();
      final isConnected = status.peerCount > 0;
      final isSynced = status.syncPercentage >= 99;
      final hasRecentBlock = status.lastBlockTime != null &&
          DateTime.now().difference(status.lastBlockTime!) < const Duration(minutes: 10);

      HealthStatus healthStatus;
      String message;

      if (!isConnected) {
        healthStatus = HealthStatus.unhealthy;
        message = 'No peers connected';
        _consecutiveFailures++;
      } else if (!isSynced) {
        healthStatus = HealthStatus.degraded;
        message = 'Sync in progress: ${status.syncPercentage}%';
        _consecutiveFailures = 0;
      } else if (!hasRecentBlock) {
        healthStatus = HealthStatus.degraded;
        message = 'No recent blocks';
        _consecutiveFailures++;
      } else {
        healthStatus = HealthStatus.healthy;
        message = 'Node healthy - ${status.peerCount} peers, height ${status.blockHeight}';
        _consecutiveFailures = 0;
      }

      final result = HealthCheckResult(
        status: healthStatus,
        message: message,
        checkedAt: DateTime.now(),
        details: {
          'peerCount': status.peerCount,
          'blockHeight': status.blockHeight,
          'syncPercentage': status.syncPercentage,
          'cpuUsage': status.cpuUsage,
          'memoryUsage': status.memoryUsage,
          'consecutiveFailures': _consecutiveFailures,
        },
      );

      _healthController?.add(result);

      if (_consecutiveFailures >= _maxConsecutiveFailures) {
        _scheduleAutoRestart();
      }
    } catch (e) {
      _consecutiveFailures++;
      final result = HealthCheckResult(
        status: HealthStatus.unhealthy,
        message: 'Health check failed: $e',
        checkedAt: DateTime.now(),
        details: {'consecutiveFailures': _consecutiveFailures, 'error': e.toString()},
      );
      _healthController?.add(result);

      if (_consecutiveFailures >= _maxConsecutiveFailures) {
        _scheduleAutoRestart();
      }
    }
  }

  void _scheduleAutoRestart() {
    _autoRestartTimer?.cancel();
    _autoRestartTimer = Timer(_autoRestartDelay, () async {
      await _performAutoRestart();
    });
  }

  Future<void> _performAutoRestart() async {
    try {
      await nodeService.stopNode();
      await Future.delayed(const Duration(seconds: 2));
      await nodeService.startNode();
      _consecutiveFailures = 0;
    } catch (e) {
      _healthController?.add(HealthCheckResult(
        status: HealthStatus.unhealthy,
        message: 'Auto-restart failed: $e',
        checkedAt: DateTime.now(),
        details: {'error': e.toString()},
      ));
    }
  }

  Future<void> dispose() async {
    await stopMonitoring();
  }
}

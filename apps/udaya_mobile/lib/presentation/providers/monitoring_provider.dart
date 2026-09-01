import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/services/monitoring_service.dart';
import 'package:udaya_mobile/services/node_service.dart';

final nodeServiceProvider = Provider<NodeService>((ref) {
  throw UnimplementedError('nodeServiceProvider must be overridden');
});

final monitoringServiceProvider = Provider<MonitoringService>((ref) {
  throw UnimplementedError('monitoringServiceProvider must be overridden');
});

final healthCheckProvider = StreamProvider<HealthCheckResult>((ref) {
  final monitoring = ref.watch(monitoringServiceProvider);
  return monitoring.healthStream;
});

final autoRestartEnabledProvider = StateProvider<bool>((ref) => true);
final consecutiveFailuresProvider = StateProvider<int>((ref) => 0);

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/domain/entities/security_entity.dart';
import 'package:udaya_mobile/services/security_service.dart';

final securityServiceProvider = Provider<SecurityService>((ref) {
  return SecurityService();
});

final floodProtectionStateProvider = Provider<FloodProtectionState>((ref) {
  return ref.watch(securityServiceProvider).state;
});

final isBlockedProvider = Provider<bool>((ref) {
  return ref.watch(floodProtectionStateProvider).isBlocked;
});

final blockedUntilProvider = Provider<DateTime?>((ref) {
  return ref.watch(floodProtectionStateProvider).blockedUntil;
});

final fuzzTestResultsProvider = FutureProvider<List<FuzzTestResult>>((ref) async {
  final service = ref.watch(securityServiceProvider);
  return service.runFuzzTests(iterations: 100);
});

final adversarialTestResultsProvider = FutureProvider<List<AdversarialTestResult>>((ref) async {
  final service = ref.watch(securityServiceProvider);
  return service.runAdversarialTests();
});

final securityAuditReportProvider = FutureProvider<SecurityAuditReport>((ref) async {
  final fuzzResults = await ref.read(fuzzTestResultsProvider.future);
  final adversarialResults = await ref.read(adversarialTestResultsProvider.future);
  final floodState = ref.read(floodProtectionStateProvider);

  final totalTests = fuzzResults.length + adversarialResults.length;
  final passedTests = fuzzResults.where((r) => r.passed).length +
      adversarialResults.where((r) => r.passed).length;
  final failedTests = totalTests - passedTests;

  final securityScore = totalTests > 0 ? (passedTests / totalTests) * 100 : 100.0;

  return SecurityAuditReport(
    generatedAt: DateTime.now(),
    totalTests: totalTests,
    passedTests: passedTests,
    failedTests: failedTests,
    fuzzResults: fuzzResults,
    adversarialResults: adversarialResults,
    floodState: floodState,
    securityScore: securityScore,
  );
});

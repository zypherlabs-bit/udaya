class FloodProtectionConfig {
  final int maxRequestsPerMinute;
  final int maxRequestsPerHour;
  final Duration blockDuration;
  final int warningThreshold;

  const FloodProtectionConfig({
    this.maxRequestsPerMinute = 60,
    this.maxRequestsPerHour = 1000,
    this.blockDuration = const Duration(minutes: 5),
    this.warningThreshold = 80,
  });
}

class RateLimitWindow {
  final int count;
  final DateTime windowStart;
  final DateTime windowEnd;

  RateLimitWindow({
    required this.count,
    required this.windowStart,
    required this.windowEnd,
  });
}

class FloodProtectionState {
  final bool isBlocked;
  final DateTime? blockedUntil;
  final int requestsInLastMinute;
  final int requestsInLastHour;
  final int totalBlockedRequests;
  final List<DateTime> recentRequests;

  FloodProtectionState({
    required this.isBlocked,
    this.blockedUntil,
    required this.requestsInLastMinute,
    required this.requestsInLastHour,
    required this.totalBlockedRequests,
    required this.recentRequests,
  });

  FloodProtectionState copyWith({
    bool? isBlocked,
    DateTime? blockedUntil,
    int? requestsInLastMinute,
    int? requestsInLastHour,
    int? totalBlockedRequests,
    List<DateTime>? recentRequests,
  }) {
    return FloodProtectionState(
      isBlocked: isBlocked ?? this.isBlocked,
      blockedUntil: blockedUntil ?? this.blockedUntil,
      requestsInLastMinute: requestsInLastMinute ?? this.requestsInLastMinute,
      requestsInLastHour: requestsInLastHour ?? this.requestsInLastHour,
      totalBlockedRequests: totalBlockedRequests ?? this.totalBlockedRequests,
      recentRequests: recentRequests ?? this.recentRequests,
    );
  }
}

class FuzzTestResult {
  final String id;
  final String testName;
  final String inputType;
  final bool passed;
  final String? errorMessage;
  final DateTime timestamp;
  final int iterations;

  FuzzTestResult({
    required this.id,
    required this.testName,
    required this.inputType,
    required this.passed,
    this.errorMessage,
    required this.timestamp,
    required this.iterations,
  });
}

class AdversarialTestResult {
  final String id;
  final String testName;
  final String attackVector;
  final bool passed;
  final String? errorMessage;
  final DateTime timestamp;
  final Map<String, dynamic> details;

  AdversarialTestResult({
    required this.id,
    required this.testName,
    required this.attackVector,
    required this.passed,
    this.errorMessage,
    required this.timestamp,
    required this.details,
  });
}

class SecurityAuditReport {
  final DateTime generatedAt;
  final int totalTests;
  final int passedTests;
  final int failedTests;
  final List<FuzzTestResult> fuzzResults;
  final List<AdversarialTestResult> adversarialResults;
  final FloodProtectionState floodState;
  final double securityScore;

  SecurityAuditReport({
    required this.generatedAt,
    required this.totalTests,
    required this.passedTests,
    required this.failedTests,
    required this.fuzzResults,
    required this.adversarialResults,
    required this.floodState,
    required this.securityScore,
  });
}

import 'dart:async';
import 'package:firebase_crashlytics/firebase_crashlytics.dart';
import 'package:flutter/foundation.dart';

class CrashReportingService {
  static bool _initialized = false;

  static Future<void> initialize() async {
    if (_initialized) return;

    try {
      if (!kDebugMode) {
        FlutterError.onError = FirebaseCrashlytics.instance.recordFlutterError;

        PlatformDispatcher.instance.onError = (error, stack) {
          FirebaseCrashlytics.instance.recordError(error, stack, fatal: true);
          return true;
        };

        await FirebaseCrashlytics.instance.setCrashlyticsCollectionEnabled(true);
        await FirebaseCrashlytics.instance.setUserIdentifier(
          await _getDeviceId(),
        );

        _initialized = true;
      }
    } catch (e) {
      debugPrint('Failed to initialize crash reporting: $e');
    }
  }

  static Future<void> logError(
    dynamic exception,
    StackTrace? stackTrace, {
    String? reason,
    bool fatal = false,
  }) async {
    try {
      await FirebaseCrashlytics.instance.recordError(
        exception,
        stackTrace,
        reason: reason,
        fatal: fatal,
      );
    } catch (e) {
      debugPrint('Failed to log error: $e');
    }
  }

  static Future<void> logMessage(String message) async {
    try {
      await FirebaseCrashlytics.instance.log(message);
    } catch (e) {
      debugPrint('Failed to log message: $e');
    }
  }

  static Future<void> setCustomKey(String key, dynamic value) async {
    try {
      if (value is String) {
        await FirebaseCrashlytics.instance.setCustomKey(key, value);
      } else if (value is int) {
        await FirebaseCrashlytics.instance.setCustomKey(key, value);
      } else if (value is bool) {
        await FirebaseCrashlytics.instance.setCustomKey(key, value);
      } else if (value is double) {
        await FirebaseCrashlytics.instance.setCustomKey(key, value);
      }
    } catch (e) {
      debugPrint('Failed to set custom key: $e');
    }
  }

  static Future<void> setUserId(String userId) async {
    try {
      await FirebaseCrashlytics.instance.setUserIdentifier(userId);
    } catch (e) {
      debugPrint('Failed to set user ID: $e');
    }
  }

  static Future<String> _getDeviceId() async {
    try {
      return 'unknown_device';
    } catch (e) {
      return 'unknown_device';
    }
  }

  static Future<void> enableCrashReporting() async {
    try {
      await FirebaseCrashlytics.instance.setCrashlyticsCollectionEnabled(true);
    } catch (e) {
      debugPrint('Failed to enable crash reporting: $e');
    }
  }

  static Future<void> disableCrashReporting() async {
    try {
      await FirebaseCrashlytics.instance.setCrashlyticsCollectionEnabled(false);
    } catch (e) {
      debugPrint('Failed to disable crash reporting: $e');
    }
  }
}

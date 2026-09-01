import 'dart:async';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/services/biometric_service.dart';

enum AppLockState { locked, unlocked, authenticating }

class AppLockNotifier extends StateNotifier<AppLockState> {
  final BiometricService _biometricService;
  Timer? _inactivityTimer;
  static const Duration _inactivityTimeout = Duration(minutes: 5);

  AppLockNotifier(this._biometricService) : super(AppLockState.unlocked) {
    _initialize();
  }

  Future<void> _initialize() async {
    final enabled = await _biometricService.isAppLockEnabled();
    if (enabled) {
      final lastAuth = await _biometricService.getLastAuthTime();
      if (lastAuth == null) {
        state = AppLockState.locked;
      } else {
        final elapsed = DateTime.now().difference(lastAuth);
        if (elapsed > _inactivityTimeout) {
          state = AppLockState.locked;
        } else {
          state = AppLockState.unlocked;
          _resetInactivityTimer();
        }
      }
    } else {
      state = AppLockState.unlocked;
    }
  }

  void _resetInactivityTimer() {
    _inactivityTimer?.cancel();
    _inactivityTimer = Timer(_inactivityTimeout, () {
      state = AppLockState.locked;
      _biometricService.clearAuthData();
    });
  }

  Future<bool> authenticate() async {
    state = AppLockState.authenticating;
    final success = await _biometricService.authenticate(
      reason: 'Authenticate to access Udaya',
    );
    if (success) {
      await _biometricService.setLastAuthTime(DateTime.now());
      state = AppLockState.unlocked;
      _resetInactivityTimer();
      return true;
    } else {
      state = AppLockState.locked;
      return false;
    }
  }

  Future<bool> enableAppLock() async {
    await _biometricService.setAppLockEnabled(true);
    final success = await authenticate();
    if (!success) {
      await _biometricService.setAppLockEnabled(false);
    }
    return success;
  }

  Future<void> disableAppLock() async {
    await _biometricService.setAppLockEnabled(false);
    await _biometricService.clearAuthData();
    state = AppLockState.unlocked;
  }

  void onAppResumed() async {
    final enabled = await _biometricService.isAppLockEnabled();
    if (!enabled) return;

    final lastAuth = await _biometricService.getLastAuthTime();
    if (lastAuth == null) {
      state = AppLockState.locked;
      return;
    }

    final elapsed = DateTime.now().difference(lastAuth);
    if (elapsed > _inactivityTimeout) {
      state = AppLockState.locked;
      await _biometricService.clearAuthData();
    } else {
      state = AppLockState.unlocked;
      _resetInactivityTimer();
    }
  }

  void onUserInteraction() {
    if (state == AppLockState.unlocked) {
      _resetInactivityTimer();
    }
  }

  @override
  void dispose() {
    _inactivityTimer?.cancel();
    super.dispose();
  }
}

final appLockProvider = StateNotifierProvider<AppLockNotifier, AppLockState>((ref) {
  return AppLockNotifier(BiometricService());
});

import 'dart:async';
import 'package:flutter/widgets.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:udaya_mobile/services/monitoring_service.dart';

class AppLifecycleManager {
  final MonitoringService monitoringService;
  final Connectivity connectivity;
  StreamSubscription<List<ConnectivityResult>>? _connectivitySubscription;
  AppLifecycleState _currentState = AppLifecycleState.inactive;

  AppLifecycleManager(this.monitoringService, this.connectivity);

  AppLifecycleState get currentState => _currentState;

  Future<void> initialize() async {
    WidgetsBinding.instance.addObserver(_LifecycleObserver(_onLifecycleChanged));
    _connectivitySubscription = connectivity.onConnectivityChanged.listen(_onConnectivityChanged);
  }

  void _onLifecycleChanged(AppLifecycleState state) {
    _currentState = state;
    if (state == AppLifecycleState.resumed) {
      _handleAppResumed();
    } else if (state == AppLifecycleState.paused) {
      _handleAppPaused();
    }
  }

  void _handleAppResumed() {
    monitoringService.startMonitoring();
  }

  void _handleAppPaused() {
    monitoringService.stopMonitoring();
  }

  Future<void> _onConnectivityChanged(List<ConnectivityResult> result) async {
    final hasConnectivity = result.any((r) => r != ConnectivityResult.none);
    if (!hasConnectivity) {
      await monitoringService.stopMonitoring();
    } else {
      await monitoringService.startMonitoring();
    }
  }

  Future<void> dispose() async {
    await _connectivitySubscription?.cancel();
    await monitoringService.dispose();
  }
}

class _LifecycleObserver extends WidgetsBindingObserver {
  final void Function(AppLifecycleState) onChanged;

  _LifecycleObserver(this.onChanged);

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    onChanged(state);
  }
}

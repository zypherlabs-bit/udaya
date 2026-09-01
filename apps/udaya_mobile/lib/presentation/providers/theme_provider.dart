import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/constants/app_constants.dart';

final themeModeProvider =
    StateNotifierProvider<ThemeModeNotifier, ThemeMode>((ref) {
  return ThemeModeNotifier();
});

class ThemeModeNotifier extends StateNotifier<ThemeMode> {
  ThemeModeNotifier() : super(ThemeMode.dark);

  void setThemeMode(ThemeMode mode) {
    state = mode;
  }

  void toggleThemeMode() {
    switch (state) {
      case ThemeMode.light:
        state = ThemeMode.dark;
      case ThemeMode.dark:
        state = ThemeMode.light;
      case ThemeMode.system:
        state = ThemeMode.dark;
    }
  }
}

enum NetworkType {
  mainnet,
  testnet,
  regtest;

  String get key {
    switch (this) {
      case NetworkType.mainnet:
        return AppConstants.mainnet;
      case NetworkType.testnet:
        return AppConstants.testnet;
      case NetworkType.regtest:
        return AppConstants.regtest;
    }
  }

  static NetworkType fromKey(String key) {
    switch (key) {
      case 'mainnet':
        return NetworkType.mainnet;
      case 'testnet':
        return NetworkType.testnet;
      case 'regtest':
        return NetworkType.regtest;
      default:
        return NetworkType.mainnet;
    }
  }
}

final currentNetworkProvider = StateProvider<NetworkType>((ref) {
  return NetworkType.mainnet;
});

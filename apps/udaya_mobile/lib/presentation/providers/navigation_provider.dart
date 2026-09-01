import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Provider for the current navigation route.
final currentRouteProvider = StateProvider<String>((ref) {
  return '/';
});

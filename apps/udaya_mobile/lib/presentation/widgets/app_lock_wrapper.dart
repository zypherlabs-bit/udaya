import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_lock_provider.dart';
import 'package:udaya_mobile/presentation/widgets/app_lock_screen.dart';

class AppLockWrapper extends ConsumerStatefulWidget {
  final Widget child;

  const AppLockWrapper({super.key, required this.child});

  @override
  ConsumerState<AppLockWrapper> createState() => _AppLockWrapperState();
}

class _AppLockWrapperState extends ConsumerState<AppLockWrapper> {
  @override
  Widget build(BuildContext context) {
    final lockState = ref.watch(appLockProvider);

    if (lockState == AppLockState.locked || lockState == AppLockState.authenticating) {
      return AppLockScreen(
        onAuthenticate: () async {
          return await ref.read(appLockProvider.notifier).authenticate();
        },
        onUnlocked: () {
          ref.read(appLockProvider.notifier).onUserInteraction();
        },
      );
    }

    return _GestureDetector(
      onInteraction: () {
        ref.read(appLockProvider.notifier).onUserInteraction();
      },
      child: widget.child,
    );
  }
}

class _GestureDetector extends StatelessWidget {
  final VoidCallback onInteraction;
  final Widget child;

  const _GestureDetector({required this.onInteraction, required this.child});

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: (_) => onInteraction(),
      onPointerMove: (_) => onInteraction(),
      child: child,
    );
  }
}

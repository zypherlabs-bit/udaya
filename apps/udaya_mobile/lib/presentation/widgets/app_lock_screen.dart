import 'dart:async';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/services/biometric_service.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';

class AppLockScreen extends ConsumerStatefulWidget {
  final VoidCallback? onUnlocked;
  final Future<bool> Function()? onAuthenticate;

  const AppLockScreen({super.key, this.onUnlocked, this.onAuthenticate});

  @override
  ConsumerState<AppLockScreen> createState() => _AppLockScreenState();
}

class _AppLockScreenState extends ConsumerState<AppLockScreen> with SingleTickerProviderStateMixin {
  bool _isAuthenticating = false;
  bool _isBiometricAvailable = false;
  String? _errorMessage;
  late AnimationController _animationController;
  late Animation<double> _fadeAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(vsync: this, duration: const Duration(milliseconds: 800));
    _fadeAnimation = Tween<double>(begin: 0, end: 1).animate(CurvedAnimation(parent: _animationController, curve: Curves.easeOutCubic));
    _animationController.forward();
    _checkBiometricAvailability();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  Future<void> _checkBiometricAvailability() async {
    final available = await BiometricService().isBiometricAvailable();
    if (mounted) {
      setState(() => _isBiometricAvailable = available);
    }
  }

  Future<void> _authenticate() async {
    setState(() {
      _isAuthenticating = true;
      _errorMessage = null;
    });

    try {
      final success = await widget.onAuthenticate?.call() ?? false;

      if (success && mounted) {
        await HapticFeedback.lightImpact();
        widget.onUnlocked?.call();
      } else if (mounted) {
        setState(() => _errorMessage = 'Authentication failed. Please try again.');
      }
    } catch (e) {
      if (mounted) {
        setState(() => _errorMessage = 'Authentication error: ${e.toString()}');
      }
    } finally {
      if (mounted) {
        setState(() => _isAuthenticating = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: Stack(
        children: [
          Positioned.fill(
            child: Container(
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                  colors: [
                    UdayaTheme.bgDark.withValues(alpha: 0.9),
                    UdayaTheme.bgDeep,
                  ],
                ),
              ),
            ),
          ),
          Center(
            child: FadeTransition(
              opacity: _fadeAnimation,
              child: GlassCard(
                padding: const EdgeInsets.all(32),
                margin: const EdgeInsets.symmetric(horizontal: 32),
                blur: false,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 64,
                      height: 64,
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          colors: [UdayaTheme.primaryColor, UdayaTheme.accentColor],
                          begin: Alignment.topLeft,
                          end: Alignment.bottomRight,
                        ),
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Icon(Icons.lock_rounded, color: UdayaTheme.textPrimary, size: 28),
                    ),
                    const SizedBox(height: 24),
                    const Text(
                      'Udaya Locked',
                      style: TextStyle(
                        fontSize: 22,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                        letterSpacing: -0.5,
                      ),
                    ),
                    const SizedBox(height: 8),
                     Text(
                       _isBiometricAvailable ? 'Authenticate to unlock' : 'Use device PIN or password to unlock',
                       style: const TextStyle(
                         fontSize: 14,
                         color: UdayaTheme.textSecondary,
                         height: 1.5,
                       ),
                       textAlign: TextAlign.center,
                     ),
                    if (_errorMessage != null) ...[
                      const SizedBox(height: 16),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        decoration: BoxDecoration(
                          color: UdayaTheme.error.withValues(alpha: 0.12),
                          borderRadius: BorderRadius.circular(10),
                          border: Border.all(color: UdayaTheme.error.withValues(alpha: 0.25), width: 1),
                        ),
                        child: Row(
                          children: [
                            Icon(Icons.error_outline_rounded, size: 16, color: UdayaTheme.error),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(
                                _errorMessage!,
                                style: const TextStyle(color: UdayaTheme.error, fontSize: 13),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                    const SizedBox(height: 24),
                    GlassButton(
                      onPressed: _isAuthenticating ? null : _authenticate,
                      backgroundColor: UdayaTheme.primaryColor,
                      width: double.infinity,
                      isLoading: _isAuthenticating,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(_isBiometricAvailable ? Icons.fingerprint_rounded : Icons.lock_rounded, size: 18),
                          const SizedBox(width: 10),
                          Text(_isAuthenticating ? 'Authenticating...' : 'Unlock', style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

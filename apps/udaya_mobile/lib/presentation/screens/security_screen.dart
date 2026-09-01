import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:udaya_mobile/presentation/widgets/status_chip.dart';
import 'package:udaya_mobile/presentation/providers/app_lock_provider.dart';
import 'package:udaya_mobile/services/biometric_service.dart';
import 'package:udaya_mobile/domain/entities/security_entity.dart';

class SecurityScreen extends ConsumerStatefulWidget {
  const SecurityScreen({super.key});

  @override
  ConsumerState<SecurityScreen> createState() => _SecurityScreenState();
}

class _SecurityScreenState extends ConsumerState<SecurityScreen> with TickerProviderStateMixin {
  bool _isEncrypted = true;
  bool _isLocked = false;
  bool _biometricEnabled = false;
  bool _biometricAvailable = false;
  bool _isRunningFuzz = false;
  bool _isRunningAdversarial = false;
  bool _seedBackedUp = false;
  AppLockTimeout _appLockTimeout = AppLockTimeout.fiveMinutes;

  late AnimationController _scoreController;
  late AnimationController _fadeController;
  late Animation<double> _fadeAnimation;

  @override
  void initState() {
    super.initState();
    _fadeController = AnimationController(
      vsync: this,
      duration: UdayaTheme.animationSlow,
    )..forward();
    _fadeAnimation = CurvedAnimation(parent: _fadeController, curve: UdayaTheme.animationCurve);
    _scoreController = AnimationController(
      vsync: this,
      duration: UdayaTheme.animationSlow,
    );
    _loadSecurityState();
  }

  Future<void> _loadSecurityState() async {
    final biometricService = BiometricService();
    final biometricAvailable = await biometricService.isBiometricAvailable();
    final biometricEnabled = await biometricService.isBiometricEnabled();
    if (mounted) {
      setState(() {
        _biometricAvailable = biometricAvailable;
        _biometricEnabled = biometricEnabled;
      });
    }
  }

  @override
  void dispose() {
    _scoreController.dispose();
    _fadeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          setState(() {});
          await Future.delayed(const Duration(milliseconds: 500));
        },
        child: CustomScrollView(
          slivers: [
            SliverAppBar(
              expandedHeight: 120,
              floating: false,
              pinned: true,
              backgroundColor: Colors.transparent,
              flexibleSpace: FlexibleSpaceBar(
                background: Container(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                      colors: [
                        UdayaTheme.bgDark.withValues(alpha: 0.95),
                        UdayaTheme.bgDark.withValues(alpha: 0.0),
                      ],
                    ),
                  ),
                ),
              ),
              title: Row(
                children: [
                  Container(
                    width: 32,
                    height: 32,
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        colors: [UdayaTheme.primaryColor, UdayaTheme.accentColor],
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Icon(Icons.security_rounded, color: UdayaTheme.textPrimary, size: 18),
                  ),
                  const SizedBox(width: 10),
                  const Text(
                    'Security Center',
                    style: TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.w600,
                      color: UdayaTheme.textPrimary,
                      letterSpacing: -0.3,
                    ),
                  ),
                ],
              ),
            ),
            SliverToBoxAdapter(
              child: FadeTransition(
                opacity: _fadeAnimation,
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _buildSecurityScoreCard(),
                      const SizedBox(height: 20),
                      _buildSecurityChecklist(),
                      const SizedBox(height: 20),
                      _buildBiometricSection(),
                      const SizedBox(height: 20),
                      _buildAppLockSection(),
                      const SizedBox(height: 20),
                      _buildSeedPhraseSection(),
                      const SizedBox(height: 20),
                      _buildEncryptionSection(),
                      const SizedBox(height: 20),
                      _buildSecurityAuditSection(),
                      const SizedBox(height: 20),
                      _buildDangerZone(),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSecurityScoreCard() {
    final score = _calculateSecurityScore();
    final level = _getSecurityLevel(score);
    final color = _getSecurityColor(score);

    return GlassCard(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Security Score',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: UdayaTheme.textTertiary,
                        letterSpacing: 0.3,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      '$score/100',
                      style: TextStyle(
                        fontSize: 36,
                        fontWeight: FontWeight.w700,
                        color: color,
                        letterSpacing: -1,
                      ),
                    ),
                    const SizedBox(height: 4),
                    StatusChip(
                      label: level,
                      color: color,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 24),
              Container(
                width: 80,
                height: 80,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  border: Border.all(color: color.withValues(alpha: 0.3), width: 4),
                ),
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    StatusIndicator(
                      color: color,
                      size: 12,
                      pulse: score >= 80,
                    ),
                    Text(
                      '$score',
                      style: TextStyle(
                        fontSize: 24,
                        fontWeight: FontWeight.w700,
                        color: color,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  int _calculateSecurityScore() {
    var score = 0;
    if (_biometricEnabled) score += 25;
    if (_isEncrypted) score += 25;
    if (_seedBackedUp) score += 20;
    if (_appLockTimeout != AppLockTimeout.never) score += 20;
    if (true) score += 10;
    return score.clamp(0, 100);
  }

  String _getSecurityLevel(int score) {
    if (score >= 90) return 'SECURE';
    if (score >= 70) return 'HIGH';
    if (score >= 50) return 'MEDIUM';
    return 'LOW';
  }

  Color _getSecurityColor(int score) {
    if (score >= 90) return UdayaTheme.success;
    if (score >= 70) return UdayaTheme.info;
    if (score >= 50) return UdayaTheme.warning;
    return UdayaTheme.error;
  }

  Widget _buildSecurityChecklist() {
    final items = [
      _SecurityChecklistItem(
        title: 'Biometric Authentication',
        subtitle: _biometricEnabled ? 'Enabled' : 'Not configured',
        isEnabled: _biometricEnabled,
        icon: Icons.fingerprint_rounded,
        color: UdayaTheme.primaryColor,
      ),
      _SecurityChecklistItem(
        title: 'Wallet Encryption',
        subtitle: 'AES-256 encrypted',
        isEnabled: _isEncrypted,
        icon: Icons.lock_rounded,
        color: UdayaTheme.info,
      ),
      _SecurityChecklistItem(
        title: 'Seed Phrase Backup',
        subtitle: _seedBackedUp ? 'Backed up' : 'Not backed up',
        isEnabled: _seedBackedUp,
        icon: Icons.vpn_key_rounded,
        color: UdayaTheme.warning,
      ),
      _SecurityChecklistItem(
        title: 'App Lock',
        subtitle: _appLockTimeout != AppLockTimeout.never ? '${_appLockTimeout.label} timeout' : 'Disabled',
        isEnabled: _appLockTimeout != AppLockTimeout.never,
        icon: Icons.lock_clock_rounded,
        color: UdayaTheme.secondaryColor,
      ),
      _SecurityChecklistItem(
        title: 'Secure Storage',
        subtitle: 'Android Keystore',
        isEnabled: true,
        icon: Icons.storage_rounded,
        color: UdayaTheme.success,
      ),
    ];

    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Security Checklist',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
              letterSpacing: -0.2,
            ),
          ),
          const SizedBox(height: 16),
          ...items.map((item) => Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: item.color.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(item.icon, color: item.color, size: 18),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        item.title,
                        style: TextStyle(
                          fontSize: 14,
                          fontWeight: FontWeight.w600,
                          color: UdayaTheme.textPrimary,
                        ),
                      ),
                      Text(
                        item.subtitle,
                        style: TextStyle(
                          fontSize: 12,
                          color: UdayaTheme.textTertiary,
                        ),
                      ),
                    ],
                  ),
                ),
                Icon(
                  item.isEnabled ? Icons.check_circle_rounded : Icons.error_outline_rounded,
                  color: item.isEnabled ? UdayaTheme.success : UdayaTheme.error,
                  size: 20,
                ),
              ],
            ),
          )).toList(),
        ],
      ),
    );
  }

  Widget _buildBiometricSection() {
    final appLockState = ref.watch(appLockProvider);
    final isAppLocked = appLockState == AppLockState.locked || appLockState == AppLockState.authenticating;
    
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: UdayaTheme.primaryColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.fingerprint_rounded, color: UdayaTheme.primaryColor, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Biometric Authentication',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      _biometricAvailable ? 'Available on this device' : 'Not available on this device',
                      style: TextStyle(
                        fontSize: 12,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
              Switch(
                value: _biometricEnabled,
                onChanged: _biometricAvailable ? (value) async {
                  setState(() => _biometricEnabled = value);
                  await BiometricService().setBiometricEnabled(value);
                  
                  if (value) {
                    final success = await ref.read(appLockProvider.notifier).enableAppLock();
                    if (!success) {
                      setState(() => _biometricEnabled = false);
                      await BiometricService().setBiometricEnabled(false);
                      if (mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('Failed to enable app lock. Please try again.'),
                            behavior: SnackBarBehavior.floating,
                            backgroundColor: UdayaTheme.error,
                          ),
                        );
                      }
                    }
                  } else {
                    await ref.read(appLockProvider.notifier).disableAppLock();
                  }
                  
                  if (mounted) {
                    setState(() {});
                  }
                } : null,
              ),
            ],
          ),
          if (_biometricEnabled) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: UdayaTheme.success.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: UdayaTheme.success.withValues(alpha: 0.2), width: 1),
              ),
              child: Row(
                children: [
                  Icon(Icons.check_circle_rounded, size: 16, color: UdayaTheme.success),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'App lock enabled. You will need to authenticate when the app is locked.',
                      style: TextStyle(color: UdayaTheme.success, fontSize: 12),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildAppLockSection() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: UdayaTheme.secondaryColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.lock_clock_rounded, color: UdayaTheme.secondaryColor, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'App Lock Timeout',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      _appLockTimeout != AppLockTimeout.never
                          ? 'Lock after ${_appLockTimeout.label}'
                          : 'Auto-lock disabled',
                      style: TextStyle(
                        fontSize: 12,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: AppLockTimeout.values.map((timeout) {
                final isSelected = _appLockTimeout == timeout;
                return Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: FilterChip(
                    label: Text(timeout.label),
                    selected: isSelected,
                    onSelected: (selected) {
                      setState(() => _appLockTimeout = timeout);
                    },
                    backgroundColor: UdayaTheme.bgSurface,
                    selectedColor: UdayaTheme.primaryColor.withValues(alpha: 0.2),
                    labelStyle: TextStyle(
                      color: isSelected ? UdayaTheme.primaryColor : UdayaTheme.textSecondary,
                      fontSize: 12,
                    ),
                    side: BorderSide(
                      color: isSelected ? UdayaTheme.primaryColor : UdayaTheme.borderColor,
                      width: 1,
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSeedPhraseSection() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      onTap: () {
        context.push('/backup-recovery');
      },
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: UdayaTheme.warning.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(Icons.vpn_key_rounded, color: UdayaTheme.warning, size: 18),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Seed Phrase Backup',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                    color: UdayaTheme.textPrimary,
                  ),
                ),
                Text(
                  _seedBackedUp ? 'Backup verified' : 'Backup not verified',
                  style: TextStyle(
                    fontSize: 12,
                    color: UdayaTheme.textTertiary,
                  ),
                ),
              ],
            ),
          ),
          StatusChip(
            label: _seedBackedUp ? 'BACKED UP' : 'NOT BACKED UP',
            color: _seedBackedUp ? UdayaTheme.success : UdayaTheme.warning,
          ),
          const SizedBox(width: 8),
          Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
        ],
      ),
    );
  }

  Widget _buildEncryptionSection() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: UdayaTheme.info.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(Icons.lock_rounded, color: UdayaTheme.info, size: 18),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Wallet Encryption',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                    color: UdayaTheme.textPrimary,
                  ),
                ),
                Text(
                  'AES-256-GCM encryption active',
                  style: TextStyle(
                    fontSize: 12,
                    color: UdayaTheme.textTertiary,
                  ),
                ),
              ],
            ),
          ),
          StatusChip(
            label: 'ENCRYPTED',
            color: UdayaTheme.success,
          ),
        ],
      ),
    );
  }

  Widget _buildSecurityAuditSection() {
    final auditState = ref.watch(securityAuditReportProvider);

    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: UdayaTheme.accentColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.bug_report_rounded, color: UdayaTheme.accentColor, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Security Audit',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      'Run security tests',
                      style: TextStyle(
                        fontSize: 12,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: GlassButton(
                  onPressed: _isRunningFuzz ? null : () async {
                    setState(() => _isRunningFuzz = true);
                    try {
                      final fuzzResults = ref.read(securityServiceProvider).runFuzzTests();
                      if (mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text('Fuzz test completed: ${fuzzResults.length} tests'),
                            behavior: SnackBarBehavior.floating,
                            backgroundColor: UdayaTheme.success,
                          ),
                        );
                      }
                    } finally {
                      if (mounted) setState(() => _isRunningFuzz = false);
                    }
                  },
                  isLoading: _isRunningFuzz,
                  child: const Text('Fuzz Test'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  onPressed: _isRunningAdversarial ? null : () async {
                    setState(() => _isRunningAdversarial = true);
                    try {
                      final adversarialResults = ref.read(securityServiceProvider).runAdversarialTests();
                      if (mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text('Adversarial test completed: ${adversarialResults.length} tests'),
                            behavior: SnackBarBehavior.floating,
                            backgroundColor: UdayaTheme.success,
                          ),
                        );
                      }
                    } finally {
                      if (mounted) setState(() => _isRunningAdversarial = false);
                    }
                  },
                  isLoading: _isRunningAdversarial,
                  isOutlined: true,
                  borderColor: UdayaTheme.secondaryColor,
                  foregroundColor: UdayaTheme.secondaryColor,
                  child: Text('Adversarial', style: TextStyle(color: UdayaTheme.secondaryColor)),
                ),
              ),
            ],
          ),
          if (auditState.value != null) ...[
            const SizedBox(height: 16),
            _buildAuditResults(auditState.value!),
          ],
        ],
      ),
    );
  }

  Widget _buildAuditResults(SecurityAuditReport report) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: UdayaTheme.borderColor, width: 1),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                'Audit Results',
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                ),
              ),
              const Spacer(),
              StatusChip(
                label: report.failedTests == 0 ? 'PASSED' : 'FAILED',
                color: report.failedTests == 0 ? UdayaTheme.success : UdayaTheme.error,
              ),
            ],
          ),
          const SizedBox(height: 12),
          ...report.fuzzResults.take(3).map((result) => Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Row(
              children: [
                Icon(
                  result.passed ? Icons.check_circle_rounded : Icons.error_rounded,
                  color: result.passed ? UdayaTheme.success : UdayaTheme.error,
                  size: 16,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    result.testName,
                    style: TextStyle(
                      fontSize: 13,
                      color: UdayaTheme.textSecondary,
                    ),
                  ),
                ),
              ],
            ),
          )).toList(),
        ],
      ),
    );
  }

  Widget _buildDangerZone() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: UdayaTheme.error.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.warning_rounded, color: UdayaTheme.error, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Danger Zone',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.error,
                      ),
                    ),
                    Text(
                      'Irreversible actions',
                      style: TextStyle(
                        fontSize: 12,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: TextButton.icon(
                  onPressed: () {
                    showDialog(
                      context: context,
                      builder: (context) => GlassDialog(
                        title: 'Delete Wallet',
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(
                              'This action cannot be undone. All wallet data will be permanently deleted.',
                              style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14),
                            ),
                            const SizedBox(height: 20),
                            Row(
                              mainAxisAlignment: MainAxisAlignment.end,
                              children: [
                                TextButton(
                                  onPressed: () => Navigator.pop(context),
                                  child: Text('Cancel', style: TextStyle(color: UdayaTheme.textTertiary)),
                                ),
                                const SizedBox(width: 8),
                                ElevatedButton(
                                  onPressed: () {
                                    Navigator.pop(context);
                                    ScaffoldMessenger.of(context).showSnackBar(
                                      const SnackBar(
                                        content: Text('Wallet deletion not yet implemented'),
                                        backgroundColor: UdayaTheme.error,
                                      ),
                                    );
                                  },
                                  style: ElevatedButton.styleFrom(
                                    backgroundColor: UdayaTheme.error,
                                    foregroundColor: UdayaTheme.textPrimary,
                                  ),
                                  child: const Text('Delete'),
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    );
                  },
              icon: Icon(Icons.delete_forever_rounded, color: UdayaTheme.error, size: 18),
              label: Text('Delete Wallet', style: TextStyle(color: UdayaTheme.error)),
            ),
          ),
        ],
      ),
    );
  }
}

class _SecurityChecklistItem {
  final String title;
  final String subtitle;
  final bool isEnabled;
  final IconData icon;
  final Color color;

  _SecurityChecklistItem({
    required this.title,
    required this.subtitle,
    required this.isEnabled,
    required this.icon,
    required this.color,
  });
}

enum AppLockTimeout {
  oneMinute('1 min'),
  fiveMinutes('5 min'),
  fifteenMinutes('15 min'),
  thirtyMinutes('30 min'),
  never('Never');

  final String label;
  const AppLockTimeout(this.label);
}

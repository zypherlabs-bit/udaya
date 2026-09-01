import 'dart:async';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/metric_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/network_status_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/animated_number.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';

class DashboardScreen extends ConsumerStatefulWidget {
  const DashboardScreen({super.key});

  @override
  ConsumerState<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends ConsumerState<DashboardScreen>
    with TickerProviderStateMixin {
  Timer? _balanceTimer;
  late AnimationController _borderController;
  late AnimationController _fadeController;
  late Animation<double> _fadeAnimation;

  bool _balanceVisible = true;
  final Map<String, bool> _visibleSections = {
    'balance': true,
    'actions': true,
    'network': true,
    'mining': true,
    'transactions': true,
  };

  @override
  void initState() {
    super.initState();
    _startBalanceAnimation();
    _borderController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 4),
    )..repeat();
    _fadeController = AnimationController(
      vsync: this,
      duration: UdayaTheme.animationSlow,
    );
    _fadeAnimation = CurvedAnimation(
      parent: _fadeController,
      curve: UdayaTheme.animationCurve,
    );
    _fadeController.forward();
  }

  void _startBalanceAnimation() {
    _balanceTimer = Timer.periodic(const Duration(seconds: 3), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _balanceTimer?.cancel();
    _borderController.dispose();
    _fadeController.dispose();
    super.dispose();
  }

  Future<void> _handleRefresh() async {
    ref.read(nodeProvider.notifier).setLoading(true);
    ref.read(walletProvider.notifier).refreshAll();
    ref.read(miningProvider.notifier).setLoading(true);
    await Future.delayed(const Duration(milliseconds: 500));
  }

  void _toggleBalanceVisibility() {
    setState(() => _balanceVisible = !_balanceVisible);
  }

  void _toggleSection(String sectionKey) {
    setState(() => _visibleSections[sectionKey] = !_visibleSections[sectionKey]!);
  }

  @override
  Widget build(BuildContext context) {
    final nodeState = ref.watch(nodeProvider);
    final walletState = ref.watch(walletProvider);
    final networkState = ref.watch(networkProvider);
    final miningState = ref.watch(miningProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: _handleRefresh,
        color: UdayaTheme.primaryColor,
        backgroundColor: UdayaTheme.bgCard,
        strokeWidth: 2.5,
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
                    child: const Icon(
                      Icons.currency_exchange_rounded,
                      color: UdayaTheme.textPrimary,
                      size: 18,
                    ),
                  ),
                  const SizedBox(width: 10),
                  const Text(
                    'Udaya',
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
                      if (_visibleSections['balance'] == true ||
                          _visibleSections['actions'] == true ||
                          _visibleSections['network'] == true ||
                          _visibleSections['mining'] == true ||
                          _visibleSections['transactions'] == true)
                        _buildSectionIndicator(),
                      const SizedBox(height: 16),
                      if (_visibleSections['balance'] == true)
                        AnimatedSize(
                          duration: UdayaTheme.animationSlow,
                          curve: UdayaTheme.animationCurve,
                          child: Column(
                            children: [
                              _buildPremiumBalanceCard(context, walletState),
                              const SizedBox(height: 20),
                            ],
                          ),
                        ),
                      if (_visibleSections['actions'] == true)
                        AnimatedSize(
                          duration: UdayaTheme.animationSlow,
                          curve: UdayaTheme.animationCurve,
                          child: Column(
                            children: [
                              _buildQuickActionsGrid(context),
                              const SizedBox(height: 20),
                            ],
                          ),
                        ),
                      if (_visibleSections['network'] == true)
                        AnimatedSize(
                          duration: UdayaTheme.animationSlow,
                          curve: UdayaTheme.animationCurve,
                          child: Column(
                            children: [
                              _buildEnhancedNetworkStatus(
                                context,
                                nodeState,
                                networkState,
                              ),
                              const SizedBox(height: 20),
                            ],
                          ),
                        ),
                      if (_visibleSections['mining'] == true &&
                          miningState.info?.isMining == true)
                        AnimatedSize(
                          duration: UdayaTheme.animationSlow,
                          curve: UdayaTheme.animationCurve,
                          child: Column(
                            children: [
                              _buildMiningStatusCard(context, miningState.info),
                              const SizedBox(height: 20),
                            ],
                          ),
                        ),
                      if (_visibleSections['transactions'] == true)
                        AnimatedSize(
                          duration: UdayaTheme.animationSlow,
                          curve: UdayaTheme.animationCurve,
                          child: _buildRecentTransactions(context, walletState),
                        ),
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

  Widget _buildSectionIndicator() {
    final sections = [
      {'key': 'balance', 'label': 'Balance', 'icon': Icons.account_balance_wallet_rounded},
      {'key': 'actions', 'label': 'Actions', 'icon': Icons.touch_app_rounded},
      {'key': 'network', 'label': 'Network', 'icon': Icons.public_rounded},
      {'key': 'mining', 'label': 'Mining', 'icon': Icons.precision_manufacturing_rounded},
      {'key': 'transactions', 'label': 'Activity', 'icon': Icons.receipt_long_rounded},
    ];

    return GlassCard(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        children: [
          const Icon(
            Icons.dashboard_customize_rounded,
            size: 16,
            color: UdayaTheme.textTertiary,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: sections.map((section) {
                  final isVisible = _visibleSections[section['key']] ?? true;
                  return Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: FilterChip(
                      label: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            section['icon'] as IconData,
                            size: 14,
                            color: isVisible
                                ? UdayaTheme.primaryColor
                                : UdayaTheme.textMuted,
                          ),
                          const SizedBox(width: 4),
                          Text(
                            section['label'] as String,
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.w600,
                              color: isVisible
                                  ? UdayaTheme.primaryColor
                                  : UdayaTheme.textMuted,
                              letterSpacing: 0.2,
                            ),
                          ),
                        ],
                      ),
                      selected: isVisible,
                      onSelected: (_) =>
                          _toggleSection(section['key'] as String),
                      backgroundColor: UdayaTheme.bgSurface.withValues(alpha: 0.5),
                      selectedColor: UdayaTheme.primaryColor.withValues(alpha: 0.15),
                      checkmarkColor: UdayaTheme.primaryColor,
                      side: BorderSide(
                        color: isVisible
                            ? UdayaTheme.primaryColor.withValues(alpha: 0.3)
                            : UdayaTheme.dividerColor,
                        width: 1,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                      ),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPremiumBalanceCard(
    BuildContext context,
    WalletState walletState,
  ) {
    final balance = walletState.balance?.totalUdya ?? 0;
    final confirmed = walletState.balance?.spendableUdya ?? 0;
    final pending = balance - confirmed;

    final displayTotal = _balanceVisible ? balance : null;
    final displayConfirmed = _balanceVisible ? confirmed : null;
    final displayPending = _balanceVisible ? pending : null;

    return AnimatedBuilder(
      animation: _borderController,
      builder: (context, child) {
        final gradientValue = _borderController.value;
        return Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(UdayaTheme.panelRadius),
            gradient: LinearGradient(
              begin: Alignment(
                -1 + gradientValue * 2,
                -1 + gradientValue * 2,
              ),
              end: Alignment(
                1 - gradientValue * 2,
                1 - gradientValue * 2,
              ),
              colors: const [
                UdayaTheme.primaryColor,
                UdayaTheme.accentColor,
                UdayaTheme.secondaryColor,
                UdayaTheme.primaryColor,
              ],
            ),
          ),
          child: Container(
            margin: const EdgeInsets.all(1.5),
            decoration: BoxDecoration(
              borderRadius:
                  BorderRadius.circular(UdayaTheme.panelRadius - 1.5),
              color: UdayaTheme.bgCard,
            ),
            child: ClipRRect(
              borderRadius:
                  BorderRadius.circular(UdayaTheme.panelRadius - 1.5),
              child: BackdropFilter(
                filter: ImageFilter.blur(sigmaX: 8, sigmaY: 8),
                child: Container(
                  padding: const EdgeInsets.all(24),
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                      colors: [
                        UdayaTheme.bgCard.withValues(alpha: 0.95),
                        UdayaTheme.bgCard.withValues(alpha: 0.85),
                      ],
                    ),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Container(
                            width: 40,
                            height: 40,
                            decoration: BoxDecoration(
                              gradient: LinearGradient(
                                colors: [
                                  UdayaTheme.primaryColor.withValues(alpha: 0.25),
                                  UdayaTheme.accentColor.withValues(alpha: 0.15),
                                ],
                                begin: Alignment.topLeft,
                                end: Alignment.bottomRight,
                              ),
                              borderRadius: BorderRadius.circular(12),
                              border: Border.all(
                                color: UdayaTheme.primaryColor.withValues(alpha: 0.2),
                                width: 1,
                              ),
                            ),
                            child: Icon(
                              Icons.account_balance_wallet_rounded,
                              color: UdayaTheme.primaryColor,
                              size: 20,
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  'Total Balance',
                                  style: TextStyle(
                                    fontSize: 13,
                                    fontWeight: FontWeight.w500,
                                    color: UdayaTheme.textTertiary,
                                    letterSpacing: 0.3,
                                  ),
                                ),
                                const SizedBox(height: 2),
                                AnimatedSwitcher(
                                  duration: UdayaTheme.animationNormal,
                                  transitionBuilder: (child, animation) {
                                    return FadeTransition(
                                      opacity: animation,
                                      child: child,
                                    );
                                  },
                                  child: displayTotal != null
                                      ? AnimatedNumber(
                                          key: ValueKey('visible-$balance'),
                                          value:
                                              '${displayTotal.toStringAsFixed(8)} UDAYA',
                                          style: const TextStyle(
                                            fontSize: 28,
                                            fontWeight: FontWeight.w700,
                                            color: UdayaTheme.textPrimary,
                                            letterSpacing: -0.8,
                                            height: 1.2,
                                          ),
                                          animate: true,
                                        )
                                      : Text(
                                          key: const ValueKey('hidden'),
                                          '•••••••• UDAYA',
                                          style: const TextStyle(
                                            fontSize: 28,
                                            fontWeight: FontWeight.w700,
                                            color: UdayaTheme.textPrimary,
                                            letterSpacing: -0.8,
                                            height: 1.2,
                                          ),
                                        ),
                                ),
                              ],
                            ),
                          ),
                          IconButton(
                            onPressed: _toggleBalanceVisibility,
                            icon: AnimatedSwitcher(
                              duration: UdayaTheme.animationFast,
                              child: Icon(
                                _balanceVisible
                                    ? Icons.visibility_rounded
                                    : Icons.visibility_off_rounded,
                                key: ValueKey(_balanceVisible),
                                color: UdayaTheme.textTertiary,
                                size: 20,
                              ),
                            ),
                            tooltip: _balanceVisible ? 'Hide balance' : 'Show balance',
                          ),
                        ],
                      ),
                      const SizedBox(height: 20),
                      Row(
                        children: [
                          Expanded(
                            child: _BalanceMetric(
                              label: 'Confirmed',
                              value: displayConfirmed != null
                                  ? displayConfirmed.toStringAsFixed(8)
                                  : '••••••••',
                              color: UdayaTheme.confirmedColor,
                              isHidden: !_balanceVisible,
                            ),
                          ),
                          Container(
                            width: 1,
                            height: 32,
                            color: UdayaTheme.dividerColor,
                          ),
                          Expanded(
                            child: _BalanceMetric(
                              label: 'Pending',
                              value: displayPending != null
                                  ? displayPending.toStringAsFixed(8)
                                  : '••••••••',
                              color: pending > 0
                                  ? UdayaTheme.syncingColor
                                  : UdayaTheme.textMuted,
                              isHidden: !_balanceVisible,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildQuickActionsGrid(BuildContext context) {
    final actions = [
      {
        'icon': Icons.arrow_upward_rounded,
        'label': 'Send',
        'color': UdayaTheme.secondaryColor,
        'route': '/send',
      },
      {
        'icon': Icons.arrow_downward_rounded,
        'label': 'Receive',
        'color': UdayaTheme.success,
        'route': '/receive',
      },
      {
        'icon': Icons.qr_code_scanner_rounded,
        'label': 'Scan QR',
        'color': UdayaTheme.accentColor,
        'route': '/qr-scanner',
      },
      {
        'icon': Icons.explore_rounded,
        'label': 'Explore',
        'color': UdayaTheme.info,
        'route': '/explorer',
      },
      {
        'icon': Icons.dns_rounded,
        'label': 'Node',
        'color': UdayaTheme.warning,
        'route': '/node',
      },
      {
        'icon': Icons.precision_manufacturing_rounded,
        'label': 'Mining',
        'color': UdayaTheme.primaryColor,
        'route': '/mining',
      },
      {
        'icon': Icons.security_rounded,
        'label': 'Security',
        'color': UdayaTheme.secondaryColor,
        'route': '/security',
      },
      {
        'icon': Icons.settings_rounded,
        'label': 'Settings',
        'color': UdayaTheme.textTertiary,
        'route': '/settings',
      },
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Quick Actions',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
            color: UdayaTheme.textPrimary,
            letterSpacing: -0.2,
          ),
        ),
        const SizedBox(height: 14),
        GridView.count(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          crossAxisCount: 2,
          mainAxisSpacing: 10,
          crossAxisSpacing: 10,
          childAspectRatio: 2.8,
          children: actions.map((action) {
            return _ActionTile(
              icon: action['icon'] as IconData,
              label: action['label'] as String,
              color: action['color'] as Color,
              onTap: () => context.push(action['route'] as String),
            );
          }).toList(),
        ),
      ],
    );
  }

  Widget _buildEnhancedNetworkStatus(
    BuildContext context,
    NodeState nodeState,
    NetworkState networkState,
  ) {
    final nodeStatus = nodeState.status;
    final isLoading = nodeState.isLoading;

    if (isLoading && nodeStatus == null) {
      return GlassCard(
        padding: const EdgeInsets.all(20),
        child: Column(
          children: [
            _ShimmerLoading(width: 120, height: 16),
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(child: _ShimmerLoading(width: double.infinity, height: 60)),
                const SizedBox(width: 12),
                Expanded(child: _ShimmerLoading(width: double.infinity, height: 60)),
              ],
            ),
          ],
        ),
      );
    }

    final networkName = networkState.network.name.toUpperCase();
    final isSyncing = networkState.isSyncing;
    final blockHeight = nodeStatus?.blockHeight ?? 0;
    final peerCount = nodeStatus?.peerCount ?? 0;
    final syncPercentage = nodeStatus?.syncPercentage ?? 0;

    return NetworkStatusCard(
      network: networkName,
      peerCount: peerCount,
      isSyncing: isSyncing,
      syncPercentage: syncPercentage,
      blockHeight: blockHeight,
      networkHeight: blockHeight,
    );
  }

  Widget _buildMiningStatusCard(
    BuildContext context,
    MiningInfo? miningInfo,
  ) {
    if (miningInfo == null) return const SizedBox.shrink();

    final hashrate = miningInfo.hashrate;
    final blocksFound = miningInfo.blocksFound;
    final threads = miningInfo.minerThreads;
    final difficulty = miningInfo.difficulty;

    String formatHashrate(double h) {
      if (h >= 1e9) return '${(h / 1e9).toStringAsFixed(2)} GH/s';
      if (h >= 1e6) return '${(h / 1e6).toStringAsFixed(2)} MH/s';
      if (h >= 1e3) return '${(h / 1e3).toStringAsFixed(2)} KH/s';
      return '${h.toStringAsFixed(2)} H/s';
    }

    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: UdayaTheme.warning,
                size: 10,
                pulse: true,
              ),
              const SizedBox(width: 10),
              const Text(
                'Mining Active',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.2,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                decoration: BoxDecoration(
                  color: UdayaTheme.warning.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                  border: Border.all(
                    color: UdayaTheme.warning.withValues(alpha: 0.25),
                    width: 1,
                  ),
                ),
                child: Text(
                  '$threads THREADS',
                  style: TextStyle(
                    color: UdayaTheme.warning,
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Hashrate',
                  value: formatHashrate(hashrate),
                  icon: Icons.speed_rounded,
                  iconColor: UdayaTheme.warning,
                  animate: true,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Blocks Found',
                  value: blocksFound.toString(),
                  icon: Icons.emoji_events_rounded,
                  iconColor: UdayaTheme.accentColor,
                  animate: true,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: MetricCard(
                  label: 'Difficulty',
                  value: _formatCompactNumber(difficulty),
                  icon: Icons.bar_chart_rounded,
                  iconColor: UdayaTheme.secondaryColor,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: MetricCard(
                  label: 'Network',
                  value: miningInfo.networkHeight.toString(),
                  icon: Icons.public_rounded,
                  iconColor: UdayaTheme.success,
                  subValue: 'Block Height',
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildRecentTransactions(BuildContext context, WalletState walletState) {
    final transactions = walletState.transactions;
    final isLoading = walletState.isLoading;
    final hasError = walletState.error != null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Text(
              'Recent Activity',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w600,
                color: UdayaTheme.textPrimary,
                letterSpacing: -0.2,
              ),
            ),
            const Spacer(),
            TextButton.icon(
              onPressed: () => context.push('/transactions'),
              icon: const Icon(
                Icons.arrow_forward_ios_rounded,
                size: 14,
                color: UdayaTheme.textTertiary,
              ),
              label: const Text(
                'View All',
                style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        if (isLoading && transactions.isEmpty)
          GlassCard(
            padding: const EdgeInsets.all(20),
            child: Column(
              children: [
                _ShimmerLoading(width: double.infinity, height: 16),
                const SizedBox(height: 12),
                _ShimmerLoading(width: double.infinity, height: 16),
                const SizedBox(height: 12),
                _ShimmerLoading(width: double.infinity, height: 16),
              ],
            ),
          )
        else if (hasError)
          GlassCard(
            padding: const EdgeInsets.all(20),
            child: Column(
              children: [
                Icon(
                  Icons.error_outline_rounded,
                  size: 32,
                  color: UdayaTheme.error.withValues(alpha: 0.7),
                ),
                const SizedBox(height: 12),
                Text(
                  'Failed to load transactions',
                  style: TextStyle(
                    color: UdayaTheme.error,
                    fontSize: 14,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  walletState.error ?? 'Unknown error',
                  style: const TextStyle(
                    color: UdayaTheme.textTertiary,
                    fontSize: 12,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 16),
                ElevatedButton.icon(
                  onPressed: () =>
                      ref.read(walletProvider.notifier).refreshAll(),
                  icon: const Icon(Icons.refresh_rounded, size: 16),
                  label: const Text('Retry'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: UdayaTheme.error.withValues(alpha: 0.15),
                    foregroundColor: UdayaTheme.error,
                    side: BorderSide(
                      color: UdayaTheme.error.withValues(alpha: 0.3),
                      width: 1,
                    ),
                  ),
                ),
              ],
            ),
          )
        else if (transactions.isEmpty)
          GlassCard(
            padding: const EdgeInsets.all(20),
            child: Column(
              children: [
                Icon(
                  Icons.inbox_rounded,
                  size: 32,
                  color: UdayaTheme.textMuted,
                ),
                const SizedBox(height: 12),
                Text(
                  'No recent transactions',
                  style: TextStyle(
                    color: UdayaTheme.textTertiary,
                    fontSize: 14,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Your transaction history will appear here',
                  style: TextStyle(
                    color: UdayaTheme.textMuted,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          )
        else
          ...transactions.take(5).map((tx) {
            final amountStr = tx.amountUdya.toStringAsFixed(8);
            final txidShort = tx.txid.length > 16
                ? '${tx.txid.substring(0, 8)}...${tx.txid.substring(tx.txid.length - 8)}'
                : tx.txid;

            TransactionCardStatus cardStatus;
            switch (tx.status) {
              case TransactionStatus.confirmed:
                cardStatus = TransactionCardStatus.confirmed;
                break;
              case TransactionStatus.pending:
                cardStatus = TransactionCardStatus.pending;
                break;
              case TransactionStatus.failed:
                cardStatus = TransactionCardStatus.failed;
                break;
            }

            return Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: TransactionCard(
                txid: txidShort,
                amount: amountStr,
                address: tx.address,
                status: cardStatus,
                timestamp: tx.timestamp,
                confirmations: tx.confirmations,
                fee: (tx.fee / AppConstants.satoshisPerCoin).toStringAsFixed(8),
                blockHeight: tx.blockHeight,
                onTap: () {},
              ),
            );
          }),
      ],
    );
  }

  String _formatCompactNumber(double value) {
    if (value >= 1e12) return '${(value / 1e12).toStringAsFixed(2)}T';
    if (value >= 1e9) return '${(value / 1e9).toStringAsFixed(2)}B';
    if (value >= 1e6) return '${(value / 1e6).toStringAsFixed(2)}M';
    if (value >= 1e3) return '${(value / 1e3).toStringAsFixed(2)}K';
    return value.toStringAsFixed(2);
  }
}

class _BalanceMetric extends StatelessWidget {
  final String label;
  final String value;
  final Color color;
  final bool isHidden;

  const _BalanceMetric({
    required this.label,
    required this.value,
    required this.color,
    this.isHidden = false,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        AnimatedSwitcher(
          duration: UdayaTheme.animationFast,
          child: Text(
            isHidden ? '••••••••' : value,
            key: ValueKey(isHidden),
            style: TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: color,
              letterSpacing: -0.3,
            ),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          label,
          style: const TextStyle(
            fontSize: 12,
            color: UdayaTheme.textTertiary,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
}

class _ActionTile extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color color;
  final VoidCallback onTap;

  const _ActionTile({
    required this.icon,
    required this.label,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 14),
      child: Row(
        children: [
          Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(icon, color: color, size: 16),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              label,
              style: TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.w600,
                color: UdayaTheme.textSecondary,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}

class _ShimmerLoading extends StatelessWidget {
  final double width;
  final double height;

  const _ShimmerLoading({
    required this.width,
    required this.height,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: UdayaTheme.animationNormal,
      width: width,
      height: height,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(6),
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            UdayaTheme.bgSurface.withValues(alpha: 0.5),
            UdayaTheme.bgSurface.withValues(alpha: 0.3),
            UdayaTheme.bgSurface.withValues(alpha: 0.5),
          ],
        ),
      ),
    );
  }
}

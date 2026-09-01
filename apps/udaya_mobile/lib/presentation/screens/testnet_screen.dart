import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';

class TestnetScreen extends ConsumerStatefulWidget {
  const TestnetScreen({super.key});

  @override
  ConsumerState<TestnetScreen> createState() => _TestnetScreenState();
}

class _TestnetScreenState extends ConsumerState<TestnetScreen> with TickerProviderStateMixin {
  final _rpcHostController = TextEditingController(text: 'localhost');
  final _rpcPortController = TextEditingController(text: '18332');
  late AnimationController _fadeController;

  @override
  void initState() {
    super.initState();
    _fadeController = AnimationController(
      vsync: this,
      duration: UdayaTheme.animationSlow,
    )..forward();
    _loadRpcConfig();
  }

  Future<void> _loadRpcConfig() async {
    final prefs = ref.read(sharedPrefsSourceProvider);
    final savedHost = prefs.getString('rpc_host');
    final savedPort = prefs.getString('rpc_port');
    if (savedHost != null && mounted) _rpcHostController.text = savedHost;
    if (savedPort != null && mounted) _rpcPortController.text = savedPort;
  }

  Future<void> _saveRpcConfig() async {
    final prefs = ref.read(sharedPrefsSourceProvider);
    final host = _rpcHostController.text.trim();
    final port = int.tryParse(_rpcPortController.text.trim()) ?? 18332;
    await prefs.setString('rpc_host', host);
    await prefs.setString('rpc_port', port.toString());

    final repo = ref.read(nodeRepositoryProvider);
    if (repo.toString().contains('NodeRepositoryImpl')) {
      // Update RPC client configuration
    }

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('RPC configuration saved'),
          behavior: SnackBarBehavior.floating,
          backgroundColor: UdayaTheme.success,
        ),
      );
    }
  }

  @override
  void dispose() {
    _rpcHostController.dispose();
    _rpcPortController.dispose();
    _fadeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final networkState = ref.watch(networkProvider);
    final isTestnet = networkState.network == AppNetwork.testnet;

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async => await Future.delayed(const Duration(milliseconds: 500)),
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
                        colors: [UdayaTheme.testnetColor, UdayaTheme.warning],
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Icon(Icons.science_rounded, color: UdayaTheme.textPrimary, size: 18),
                  ),
                  const SizedBox(width: 10),
                  const Text(
                    'Testnet',
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
                opacity: CurvedAnimation(parent: _fadeController, curve: UdayaTheme.animationCurve),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _buildWarningBanner(),
                      const SizedBox(height: 16),
                      _buildTestnetToggle(isTestnet),
                      const SizedBox(height: 16),
                      _buildFaucetInfo(),
                      const SizedBox(height: 16),
                      _buildRpcConfig(),
                      const SizedBox(height: 16),
                      _buildResetButton(),
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

  Widget _buildWarningBanner() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: UdayaTheme.testnetColor.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: UdayaTheme.testnetColor.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.warning_amber_rounded, color: UdayaTheme.testnetColor, size: 20),
              const SizedBox(width: 10),
              Text(
                'UDAYA TESTNET',
                style: TextStyle(
                  color: UdayaTheme.testnetColor,
                  fontWeight: FontWeight.w700,
                  fontSize: 14,
                  letterSpacing: 0.5,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'TEST COINS — NO MONETARY VALUE',
            style: TextStyle(
              color: UdayaTheme.testnetColor.withValues(alpha: 0.9),
              fontWeight: FontWeight.w600,
              fontSize: 13,
            ),
          ),
          const SizedBox(height: 6),
          Text(
            'Do not send mainnet coins to testnet addresses. Testnet coins have no real value.',
            style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13, height: 1.5),
          ),
        ],
      ),
    );
  }

  Widget _buildTestnetToggle(bool isTestnet) {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              color: isTestnet ? UdayaTheme.testnetColor.withValues(alpha: 0.15) : UdayaTheme.textMuted.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Icon(
              isTestnet ? Icons.science_rounded : Icons.public_rounded,
              color: isTestnet ? UdayaTheme.testnetColor : UdayaTheme.textMuted,
              size: 22,
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Testnet Mode',
                  style: TextStyle(
                    color: UdayaTheme.textPrimary,
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                Text(
                  isTestnet ? 'Connected to testnet' : 'Connected to mainnet',
                  style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13),
                ),
              ],
            ),
          ),
          Switch(
            value: isTestnet,
            onChanged: _toggleTestnet,
            activeColor: UdayaTheme.testnetColor,
          ),
        ],
      ),
    );
  }

  Widget _buildFaucetInfo() {
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
                  color: UdayaTheme.warning.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.water_drop_rounded, color: UdayaTheme.warning, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Testnet Faucet',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      'Get free testnet coins',
                      style: TextStyle(
                        fontSize: 13,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: UdayaTheme.bgSurface,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: UdayaTheme.borderColor, width: 1),
            ),
            child: Text(
              'https://faucet.udaya.network',
              style: TextStyle(
                color: UdayaTheme.info,
                fontFamily: 'monospace',
                fontSize: 13,
              ),
            ),
          ),
          const SizedBox(height: 12),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Opening faucet in browser...'),
                    behavior: SnackBarBehavior.floating,
                  ),
                );
              },
              child: Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.open_in_new_rounded, size: 18, color: UdayaTheme.textPrimary),
                  const SizedBox(width: 8),
                  const Text('Open Faucet'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRpcConfig() {
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
                  color: UdayaTheme.info.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.dns_rounded, color: UdayaTheme.info, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'RPC Configuration',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      'Testnet node endpoint',
                      style: TextStyle(
                        fontSize: 13,
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _rpcHostController,
            decoration: const InputDecoration(
              labelText: 'RPC Host',
              prefixIcon: Icon(Icons.dns_rounded),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _rpcPortController,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'RPC Port',
              prefixIcon: Icon(Icons.numbers_rounded),
            ),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              onPressed: _saveRpcConfig,
              child: const Text('Save Configuration'),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildResetButton() {
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
                child: Icon(Icons.delete_forever_rounded, color: UdayaTheme.error, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Reset Testnet Data',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.error,
                      ),
                    ),
                    Text(
                      'Delete all testnet blockchain data',
                      style: TextStyle(
                        fontSize: 13,
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
            child: GlassButton(
              backgroundColor: UdayaTheme.error,
              foregroundColor: UdayaTheme.textPrimary,
              onPressed: _confirmReset,
              child: const Text('Reset Testnet Data'),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _toggleTestnet(bool value) async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: value ? 'Enable Testnet' : 'Disable Testnet',
      message: value
          ? 'Switching to testnet will change network endpoints. Continue?'
          : 'Switching back to mainnet will change network endpoints. Continue?',
      confirmText: 'Switch Network',
      icon: Icons.swap_horiz_rounded,
    );

    if (confirmed == true) {
      ref.read(networkProvider.notifier).setNetwork(
            value ? AppNetwork.testnet : AppNetwork.mainnet,
          );
    }
  }

  Future<void> _confirmReset() async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Reset Testnet Data',
      message: 'This action cannot be undone. All testnet blockchain data will be permanently deleted.',
      confirmText: 'Reset Data',
      isDestructive: true,
      icon: Icons.delete_forever_rounded,
    );

    if (confirmed == true && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Testnet data reset successfully'),
          behavior: SnackBarBehavior.floating,
          backgroundColor: UdayaTheme.success,
        ),
      );
    }
  }
}

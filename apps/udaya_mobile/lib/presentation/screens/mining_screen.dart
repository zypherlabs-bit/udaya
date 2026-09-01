import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';

class MiningScreen extends ConsumerStatefulWidget {
  const MiningScreen({super.key});

  @override
  ConsumerState<MiningScreen> createState() => _MiningScreenState();
}

class _MiningScreenState extends ConsumerState<MiningScreen> {
  int _threadCount = 1;
  bool _showBatteryWarning = true;

  @override
  Widget build(BuildContext context) {
    final miningState = ref.watch(miningProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          ref.read(miningProvider.notifier);
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
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        colors: [UdayaTheme.warning, UdayaTheme.error],
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Icon(Icons.bolt_rounded, color: UdayaTheme.textPrimary, size: 16),
                  ),
                  const SizedBox(width: 8),
                  const Text(
                    'Mining',
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
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (_showBatteryWarning) _buildBatteryWarning(),
                    _buildMiningToggle(miningState),
                    const SizedBox(height: 16),
                    _buildMiningStats(miningState),
                    const SizedBox(height: 16),
                    _buildThreadSelector(),
                    const SizedBox(height: 16),
                    _buildMiningAddress(),
                    const SizedBox(height: 20),
                    _buildMiningInfo(),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBatteryWarning() {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: UdayaTheme.warning.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: UdayaTheme.warning.withValues(alpha: 0.3)),
      ),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: UdayaTheme.warning.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(Icons.battery_alert_rounded, color: UdayaTheme.warning, size: 18),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Battery & Thermal Warning',
                  style: TextStyle(
                    color: UdayaTheme.warning,
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  'Mining uses significant CPU and may drain battery or cause overheating.',
                  style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 12),
                ),
              ],
            ),
          ),
          IconButton(
            icon: Icon(Icons.close_rounded, size: 18, color: UdayaTheme.warning),
            onPressed: () => setState(() => _showBatteryWarning = false),
          ),
        ],
      ),
    );
  }

  Widget _buildMiningToggle(MiningState miningState) {
    final isMining = miningState.info?.isMining ?? false;

    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: isMining
                    ? [UdayaTheme.success.withValues(alpha: 0.2), UdayaTheme.accentColor.withValues(alpha: 0.1)]
                    : [UdayaTheme.textMuted.withValues(alpha: 0.1), UdayaTheme.textMuted.withValues(alpha: 0.05)],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Icon(
              isMining ? Icons.power_settings_new_rounded : Icons.power_off_rounded,
              color: isMining ? UdayaTheme.success : UdayaTheme.textMuted,
              size: 22,
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Mining Status',
                  style: TextStyle(
                    color: UdayaTheme.textPrimary,
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  isMining ? 'Mining in progress' : 'Mining stopped',
                  style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13),
                ),
              ],
            ),
          ),
          Switch(
            value: isMining,
            onChanged: (value) => _toggleMining(value),
            activeColor: UdayaTheme.success,
          ),
        ],
      ),
    );
  }

  Widget _buildMiningStats(MiningState miningState) {
    final info = miningState.info;

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
                child: Icon(Icons.speed_rounded, color: UdayaTheme.warning, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Hashrate',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: UdayaTheme.textTertiary,
                        letterSpacing: 0.3,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      _formatHashrate(info?.hashrate ?? 0),
                      style: TextStyle(
                        fontSize: 22,
                        fontWeight: FontWeight.w700,
                        color: UdayaTheme.textPrimary,
                        letterSpacing: -0.5,
                      ),
                    ),
                  ],
                ),
              ),
              if (info?.isMining == true)
                StatusIndicator(
                  color: UdayaTheme.success,
                  size: 10,
                  pulse: true,
                ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: _MiningMetric(
                  label: 'Difficulty',
                  value: info?.difficulty != null ? _formatNumber(info!.difficulty!.toInt()) : 'N/A',
                  color: UdayaTheme.info,
                ),
              ),
              Container(width: 1, height: 32, color: UdayaTheme.dividerColor),
              Expanded(
                child: _MiningMetric(
                  label: 'Blocks Found',
                  value: '${info?.blocksFound ?? 0}',
                  color: UdayaTheme.success,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: _MiningMetric(
                  label: 'Network Height',
                  value: info?.networkHeight?.toString() ?? 'N/A',
                  color: UdayaTheme.secondaryColor,
                ),
              ),
              Container(width: 1, height: 32, color: UdayaTheme.dividerColor),
              Expanded(
                child: _MiningMetric(
                  label: 'Threads',
                  value: '$_threadCount',
                  color: UdayaTheme.accentColor,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildThreadSelector() {
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
                child: Icon(Icons.memory_rounded, color: UdayaTheme.secondaryColor, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Mining Threads',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      'Threads: $_threadCount',
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
          SliderTheme(
            data: SliderTheme.of(context).copyWith(
              activeTrackColor: UdayaTheme.secondaryColor,
              inactiveTrackColor: UdayaTheme.textMuted.withValues(alpha: 0.2),
              thumbColor: UdayaTheme.secondaryColor,
              overlayColor: UdayaTheme.secondaryGlow,
              trackHeight: 4,
              thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 10),
            ),
            child: Slider(
              value: _threadCount.toDouble(),
              min: 1,
              max: 8,
              divisions: 7,
              label: '$_threadCount',
              onChanged: (value) {
                setState(() => _threadCount = value.round());
              },
            ),
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('1', style: TextStyle(color: UdayaTheme.textMuted, fontSize: 12)),
              Text('8', style: TextStyle(color: UdayaTheme.textMuted, fontSize: 12)),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildMiningAddress() {
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
                child: Icon(Icons.account_balance_wallet_rounded, color: UdayaTheme.info, size: 18),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Mining Address',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: UdayaTheme.textPrimary,
                      ),
                    ),
                    Text(
                      'Rewards destination',
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
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            decoration: BoxDecoration(
              color: UdayaTheme.bgSurface,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: UdayaTheme.borderColor, width: 1),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    'U1qr39...xYz9aB (Default)',
                    style: TextStyle(
                      color: UdayaTheme.textPrimary,
                      fontFamily: 'monospace',
                      fontSize: 13,
                    ),
                  ),
                ),
                GlassIconButton(
                  icon: Icons.copy_rounded,
                  iconColor: UdayaTheme.textTertiary,
                  size: 32,
                  iconSize: 16,
                  onPressed: () {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Address copied'),
                        behavior: SnackBarBehavior.floating,
                      ),
                    );
                  },
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMiningInfo() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Mining Information',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 12),
          _InfoRow(
            icon: Icons.info_outline_rounded,
            title: 'Consensus',
            value: 'Proof of Work',
            color: UdayaTheme.info,
          ),
          const SizedBox(height: 10),
          _InfoRow(
            icon: Icons.timer_rounded,
            title: 'Block Time',
            value: '10 minutes',
            color: UdayaTheme.secondaryColor,
          ),
          const SizedBox(height: 10),
          _InfoRow(
            icon: Icons.monetization_on_rounded,
            title: 'Block Reward',
            value: '50 UDAYA',
            color: UdayaTheme.success,
          ),
        ],
      ),
    );
  }

  String _formatHashrate(double hashrate) {
    if (hashrate == 0) return '0 H/s';
    if (hashrate < 1000) return '${hashrate.toStringAsFixed(1)} H/s';
    if (hashrate < 1000000) return '${(hashrate / 1000).toStringAsFixed(2)} KH/s';
    if (hashrate < 1000000000) return '${(hashrate / 1000000).toStringAsFixed(2)} MH/s';
    return '${(hashrate / 1000000000).toStringAsFixed(2)} GH/s';
  }

  String _formatNumber(int value) {
    if (value >= 1000000000) return '${(value / 1000000000).toStringAsFixed(2)}B';
    if (value >= 1000000) return '${(value / 1000000).toStringAsFixed(2)}M';
    if (value >= 1000) return '${(value / 1000).toStringAsFixed(2)}K';
    return value.toString();
  }

  Future<void> _toggleMining(bool value) async {
    if (value) {
      final confirmed = await ConfirmDialog.show(
        context,
        title: 'Start Mining',
        message: 'Mining uses significant CPU resources and may drain your battery. Continue?',
        confirmText: 'Start Mining',
        icon: Icons.play_arrow_rounded,
      );
      if (confirmed != true) return;
    }

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(value ? 'Mining started' : 'Mining stopped'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}

class _MiningMetric extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _MiningMetric({required this.label, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          value,
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: color,
            letterSpacing: -0.2,
          ),
        ),
        const SizedBox(height: 4),
        Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: UdayaTheme.textTertiary,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
}

class _InfoRow extends StatelessWidget {
  final IconData icon;
  final String title;
  final String value;
  final Color color;

  const _InfoRow({required this.icon, required this.title, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 16, color: color),
        const SizedBox(width: 10),
        Text(
          title,
          style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 13),
        ),
        const Spacer(),
        Text(
          value,
          style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 13, fontWeight: FontWeight.w500),
        ),
      ],
    );
  }
}

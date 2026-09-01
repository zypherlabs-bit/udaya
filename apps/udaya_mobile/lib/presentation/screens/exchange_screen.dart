import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/exchange_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';

class ExchangeScreen extends ConsumerWidget {
  const ExchangeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final rpcAsync = ref.watch(rpcConnectionStatusProvider);
    final ratesAsync = ref.watch(exchangeRatesProvider);
    final stats = ref.watch(depositWithdrawalStatsProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(rpcConnectionStatusProvider);
          ref.invalidate(exchangeRatesProvider);
        },
        child: CustomScrollView(
          slivers: [
            const SliverAppBar(
              pinned: true,
              floating: false,
              backgroundColor: Colors.transparent,
              elevation: 0,
              title: Text(
                'Exchange',
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.3,
                ),
              ),
            ),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    rpcAsync.when(
                        data: (status) => _buildRpcCard(context, status),
                        loading: () => const LoadingIndicator(message: 'Checking RPC...'),
                        error: (e, _) => ErrorDisplay(message: 'RPC check failed: $e'),
                      ),
                      const SizedBox(height: 16),
                      ratesAsync.when(
                        data: (rates) => _buildRatesCard(context, rates),
                        loading: () => const LoadingIndicator(message: 'Loading rates...'),
                        error: (e, _) => ErrorDisplay(message: 'Rates failed: $e'),
                      ),
                      const SizedBox(height: 16),
                      _buildStatsCard(context, stats),
                      const SizedBox(height: 16),
                      _buildActionsCard(context, ref),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildRpcCard(BuildContext context, RpcConnectionStatus status) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'RPC Connection',
                style: TextStyle(
                  color: UdayaTheme.textPrimary,
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                ),
              ),
              Icon(
                status.isConnected ? Icons.wifi : Icons.wifi_off,
                color: status.isConnected ? UdayaTheme.success : UdayaTheme.error,
              ),
            ],
          ),
          const SizedBox(height: 12),
          _buildMetricRow('Endpoint', status.endpoint ?? 'Not configured'),
          _buildMetricRow('Status', status.isConnected ? 'Connected' : 'Disconnected'),
          _buildMetricRow('Latency', '${status.latencyMs}ms'),
          _buildMetricRow('Block Height', '${status.blockHeight}'),
          _buildMetricRow('Last Checked', _formatTime(status.lastChecked)),
          if (status.errors.isNotEmpty) ...[
            const SizedBox(height: 8),
            ...status.errors.map((e) => Padding(
              padding: const EdgeInsets.symmetric(vertical: 2),
              child: Row(
                children: [
                  Icon(Icons.error, size: 14, color: UdayaTheme.error),
                  const SizedBox(width: 8),
                  Expanded(child: Text(e, style: const TextStyle(color: UdayaTheme.error, fontSize: 12))),
                ],
              ),
            )),
          ],
        ],
      ),
    );
  }

  Widget _buildRatesCard(BuildContext context, List<ExchangeRate> rates) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Exchange Rates',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          ...rates.map((rate) => Padding(
            padding: const EdgeInsets.symmetric(vertical: 6),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  rate.pair,
                  style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 14),
                ),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      rate.price.toStringAsFixed(6),
                      style: const TextStyle(color: UdayaTheme.textPrimary, fontSize: 14, fontWeight: FontWeight.w600),
                    ),
                    Text(
                      '${rate.change24h >= 0 ? '+' : ''}${rate.change24h.toStringAsFixed(2)}%',
                      style: TextStyle(
                        color: rate.change24h >= 0 ? UdayaTheme.success : UdayaTheme.error,
                        fontSize: 12,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          )),
        ],
      ),
    );
  }

  Widget _buildStatsCard(BuildContext context, DepositWithdrawalStats stats) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Transfer Statistics',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          _buildMetricRow('Total Deposits', '${stats.totalDeposits}'),
          _buildMetricRow('Total Withdrawals', '${stats.totalWithdrawals}'),
          _buildMetricRow('Total Deposited', '${stats.totalDeposited.toStringAsFixed(4)} UDYA'),
          _buildMetricRow('Total Withdrawn', '${stats.totalWithdrawn.toStringAsFixed(4)} UDYA'),
          _buildMetricRow('Pending Deposits', '${stats.pendingDeposits}'),
          _buildMetricRow('Pending Withdrawals', '${stats.pendingWithdrawals}'),
        ],
      ),
    );
  }

  Widget _buildActionsCard(BuildContext context, WidgetRef ref) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Exchange Actions',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              backgroundColor: UdayaTheme.primaryColor,
              foregroundColor: UdayaTheme.textPrimary,
              onPressed: () async {
                final service = ref.read(exchangeServiceProvider);
                final address = await service.generateDepositAddress('Exchange Deposit');
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('Deposit address: ${address.address}'),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                }
              },
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.account_balance_wallet, size: 18),
                  SizedBox(width: 8),
                  Text('Generate Deposit Address'),
                ],
              ),
            ),
          ),
          const SizedBox(height: 8),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              isOutlined: true,
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Withdrawal flow coming soon'),
                    behavior: SnackBarBehavior.floating,
                  ),
                );
              },
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.send, size: 18),
                  SizedBox(width: 8),
                  Text('Request Withdrawal'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMetricRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 14)),
          Text(value, style: const TextStyle(color: UdayaTheme.textPrimary, fontSize: 14, fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);
    if (diff.inSeconds < 60) return '${diff.inSeconds}s ago';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    return '${diff.inHours}h ago';
  }
}

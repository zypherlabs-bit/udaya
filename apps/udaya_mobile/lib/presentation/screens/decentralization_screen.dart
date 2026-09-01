import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/decentralization_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';

class DecentralizationScreen extends ConsumerWidget {
  const DecentralizationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final metricsAsync = ref.watch(decentralizationMetricsProvider);
    final alertsAsync = ref.watch(decentralizationAlertsProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(decentralizationMetricsProvider);
          ref.invalidate(decentralizationAlertsProvider);
        },
        child: CustomScrollView(
          slivers: [
            const SliverAppBar(
              pinned: true,
              floating: false,
              backgroundColor: Colors.transparent,
              elevation: 0,
              title: Text(
                'Decentralization',
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
                    metricsAsync.when(
                      data: (metrics) => _buildMetrics(context, metrics),
                      loading: () => const LoadingIndicator(message: 'Calculating decentralization...'),
                      error: (e, _) => ErrorDisplay(message: 'Failed to load metrics: $e'),
                    ),
                    const SizedBox(height: 24),
                    alertsAsync.when(
                      data: (alerts) => _buildAlerts(context, alerts),
                      loading: () => const SizedBox.shrink(),
                      error: (_, __) => const SizedBox.shrink(),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetrics(BuildContext context, DecentralizationMetrics metrics) {
    final nakamoto = metrics.nakamotoCoefficient;
    final diversity = metrics.peerDiversity;
    final score = metrics.decentralizationScore;
    final scoreColor = score >= 0.7
        ? UdayaTheme.success
        : score >= 0.4
            ? UdayaTheme.warning
            : UdayaTheme.error;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildScoreCard(context, score, scoreColor),
        const SizedBox(height: 16),
        _buildNakamotoCard(context, nakamoto),
        const SizedBox(height: 16),
        _buildDiversityCard(context, diversity),
        const SizedBox(height: 16),
        _buildRegionCard(context, metrics),
      ],
    );
  }

  Widget _buildScoreCard(BuildContext context, double score, Color scoreColor) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Decentralization Score',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Text(
                '${(score * 100).toInt()}%',
                style: TextStyle(
                  color: scoreColor,
                  fontSize: 32,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const Spacer(),
              Container(
                width: 60,
                height: 60,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  border: Border.all(color: scoreColor, width: 4),
                ),
                child: CircularProgressIndicator(
                  value: score,
                  strokeWidth: 4,
                  valueColor: AlwaysStoppedAnimation<Color>(scoreColor),
                  backgroundColor: Colors.transparent,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildNakamotoCard(BuildContext context, NakamotoCoefficient nakamoto) {
    final isHealthy = nakamoto.value >= 5;
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Nakamoto Coefficient',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Text(
                '${nakamoto.value}',
                style: TextStyle(
                  color: isHealthy ? UdayaTheme.success : UdayaTheme.warning,
                  fontSize: 24,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(width: 8),
              Text(
                isHealthy ? 'Healthy' : 'Below target (5)',
                style: TextStyle(
                  color: UdayaTheme.textSecondary,
                  fontSize: 14,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDiversityCard(BuildContext context, PeerDiversityMetrics diversity) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Peer Diversity',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 12),
          _buildMetricRow('Total Peers', '${diversity.totalPeers}'),
          _buildMetricRow('Countries', '${diversity.uniqueCountries}'),
          _buildMetricRow('ASNs', '${diversity.uniqueAsns}'),
          _buildMetricRow('Providers', '${diversity.uniqueProviders}'),
        ],
      ),
    );
  }

  Widget _buildRegionCard(BuildContext context, DecentralizationMetrics metrics) {
    final regions = <Widget>[];

    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Regional Distribution',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 12),
          ...regions,
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
          Text(
            label,
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          Text(
            value,
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 14,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAlerts(BuildContext context, List<DecentralizationAlert> alerts) {
    if (alerts.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Alerts',
          style: TextStyle(
            color: UdayaTheme.textPrimary,
            fontSize: 18,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 8),
        ...alerts.map((alert) => GlassCard(
          color: alert.severity == 'critical'
              ? UdayaTheme.error.withValues(alpha: 0.1)
              : UdayaTheme.warning.withValues(alpha: 0.1),
          padding: const EdgeInsets.all(12),
          margin: const EdgeInsets.only(bottom: 8),
          child: Row(
            children: [
              Icon(
                alert.severity == 'critical'
                    ? Icons.error
                    : Icons.warning,
                color: alert.severity == 'critical'
                    ? UdayaTheme.error
                    : UdayaTheme.warning,
              ),
              const SizedBox(width: 8),
               Expanded(
                 child: Text(
                   alert.message,
                   style: TextStyle(
                     color: UdayaTheme.textPrimary,
                     fontSize: 14,
                   ),
                 ),
               ),
             ],
           ),
          ),
        ),
      ],
    );
  }
}

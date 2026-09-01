import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class BalanceCard extends StatelessWidget {
  final double balance;
  final String symbol;
  final String? fiatEquivalent;
  final VoidCallback? onTap;
  final bool isLoading;

  const BalanceCard({
    super.key,
    required this.balance,
    this.symbol = 'UDAYA',
    this.fiatEquivalent,
    this.onTap,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              const Text(
                'Total Balance',
                style: TextStyle(
                  color: UdayaTheme.textSecondary,
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 8),
              if (isLoading)
                const SizedBox(
                  height: 48,
                  child: Center(child: CircularProgressIndicator()),
                )
              else
                FittedBox(
                  fit: BoxFit.scaleDown,
                  child: Text(
                    '${balance.toStringAsFixed(8)} $symbol',
                    style: const TextStyle(
                      color: UdayaTheme.textPrimary,
                      fontSize: 36,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              if (fiatEquivalent != null) ...[
                const SizedBox(height: 4),
                Text(
                  fiatEquivalent!,
                  style: const TextStyle(
                    color: UdayaTheme.textTertiary,
                    fontSize: 16,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

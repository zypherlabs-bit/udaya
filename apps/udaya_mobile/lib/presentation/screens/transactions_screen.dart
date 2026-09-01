import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';
import 'package:udaya_mobile/domain/entities/wallet_entity.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_dialog.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/metric_card.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';

class TransactionsScreen extends ConsumerStatefulWidget {
  const TransactionsScreen({super.key});

  @override
  ConsumerState<TransactionsScreen> createState() => _TransactionsScreenState();
}

class _TransactionsScreenState extends ConsumerState<TransactionsScreen> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final TextEditingController _searchController = TextEditingController();
  TransactionDirection? _selectedDirection;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final walletState = ref.watch(walletProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      appBar: GlassAppBar(
        title: 'Transactions',
        backgroundColor: UdayaTheme.bgDark,
      ),
      body: Column(
        children: [
          _buildFilterBar(),
          Expanded(
            child: walletState.isLoading
                ? const LoadingIndicator(message: 'Loading transactions...')
                : walletState.error != null
                    ? ErrorDisplay(
                        message: walletState.error!,
                        onRetry: () => ref.read(walletProvider.notifier).setLoading(true),
                      )
                    : RefreshIndicator(
                        onRefresh: () async {
                          ref.read(walletProvider.notifier).setLoading(true);
                        },
                        child: TabBarView(
                          controller: _tabController,
                          children: [
                            _buildTransactionList(_filterAll(walletState.transactions)),
                            _buildTransactionList(_filterTransactions(walletState.transactions, TransactionDirection.sent)),
                            _buildTransactionList(_filterTransactions(walletState.transactions, TransactionDirection.received)),
                            _buildTransactionList(_filterPending(walletState.transactions)),
                          ],
                        ),
                      ),
          ),
        ],
      ),
    );
  }

  Widget _buildFilterBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: UdayaTheme.bgDark.withValues(alpha: 0.8),
        border: const Border(bottom: BorderSide(color: UdayaTheme.dividerColor, width: 1)),
      ),
      child: Row(
        children: [
          Expanded(
            child: Container(
              height: 40,
              decoration: BoxDecoration(
                color: UdayaTheme.bgSurface.withValues(alpha: 0.6),
                borderRadius: BorderRadius.circular(UdayaTheme.inputRadius),
                border: Border.all(color: UdayaTheme.borderColor, width: 1),
              ),
              child: TextField(
                controller: _searchController,
                style: const TextStyle(color: UdayaTheme.textPrimary, fontSize: 13),
                decoration: InputDecoration(
                  hintText: 'Search transactions...',
                  hintStyle: const TextStyle(color: UdayaTheme.textMuted, fontSize: 13),
                  prefixIcon: const Icon(Icons.search_rounded, size: 18, color: UdayaTheme.textTertiary),
                  suffixIcon: _searchController.text.isNotEmpty
                      ? GlassIconButton(
                          icon: Icons.close_rounded,
                          iconColor: UdayaTheme.textTertiary,
                          size: 28,
                          iconSize: 14,
                          onPressed: () {
                            _searchController.clear();
                            setState(() {});
                          },
                        )
                      : null,
                  border: InputBorder.none,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                ),
                onChanged: (_) => setState(() {}),
              ),
            ),
          ),
          const SizedBox(width: 10),
          GlassIconButton(
            icon: Icons.filter_list_rounded,
            iconColor: _selectedDirection != null ? UdayaTheme.secondaryColor : UdayaTheme.textSecondary,
            size: 40,
            iconSize: 18,
            onPressed: _showFilterDialog,
          ),
        ],
      ),
    );
  }

  void _showFilterDialog() {
    GlassDialog.show(
      context: context,
      title: 'Filter Transactions',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _FilterOption(
            label: 'All Directions',
            icon: Icons.swap_horiz_rounded,
            isSelected: _selectedDirection == null,
            onTap: () {
              setState(() => _selectedDirection = null);
              Navigator.pop(context);
            },
          ),
          const SizedBox(height: 8),
          _FilterOption(
            label: 'Sent Only',
            icon: Icons.arrow_upward_rounded,
            isSelected: _selectedDirection == TransactionDirection.sent,
            onTap: () {
              setState(() => _selectedDirection = TransactionDirection.sent);
              Navigator.pop(context);
            },
          ),
          const SizedBox(height: 8),
          _FilterOption(
            label: 'Received Only',
            icon: Icons.arrow_downward_rounded,
            isSelected: _selectedDirection == TransactionDirection.received,
            onTap: () {
              setState(() => _selectedDirection = TransactionDirection.received);
              Navigator.pop(context);
            },
          ),
        ],
      ),
    );
  }

  List<WalletTransaction> _filterAll(List<WalletTransaction> transactions) {
    var filtered = transactions;
    if (_selectedDirection != null) {
      filtered = filtered.where((tx) => tx.direction == _selectedDirection).toList();
    }
    if (_searchController.text.isNotEmpty) {
      final query = _searchController.text.toLowerCase();
      filtered = filtered.where((tx) {
        return tx.txid.toLowerCase().contains(query) ||
            (tx.address?.toLowerCase().contains(query) ?? false) ||
            tx.amountUdya.toString().contains(query);
      }).toList();
    }
    return filtered;
  }

  List<WalletTransaction> _filterTransactions(List<WalletTransaction> transactions, TransactionDirection direction) {
    var filtered = transactions.where((tx) => tx.direction == direction).toList();
    if (_searchController.text.isNotEmpty) {
      final query = _searchController.text.toLowerCase();
      filtered = filtered.where((tx) {
        return tx.txid.toLowerCase().contains(query) ||
            (tx.address?.toLowerCase().contains(query) ?? false) ||
            tx.amountUdya.toString().contains(query);
      }).toList();
    }
    return filtered;
  }

  List<WalletTransaction> _filterPending(List<WalletTransaction> transactions) {
    var filtered = transactions.where((tx) => tx.status == TransactionStatus.pending).toList();
    if (_searchController.text.isNotEmpty) {
      final query = _searchController.text.toLowerCase();
      filtered = filtered.where((tx) {
        return tx.txid.toLowerCase().contains(query) ||
            (tx.address?.toLowerCase().contains(query) ?? false) ||
            tx.amountUdya.toString().contains(query);
      }).toList();
    }
    return filtered;
  }

  Widget _buildTransactionList(List<WalletTransaction> transactions) {
    if (transactions.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: UdayaTheme.bgCard,
                borderRadius: BorderRadius.circular(20),
                border: Border.all(color: UdayaTheme.borderColor, width: 1),
              ),
              child: Icon(Icons.receipt_long_outlined, size: 32, color: UdayaTheme.textMuted),
            ),
            const SizedBox(height: 20),
            const Text(
              'No transactions found',
              style: TextStyle(
                color: UdayaTheme.textSecondary,
                fontSize: 16,
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Your transactions will appear here',
              style: TextStyle(
                color: UdayaTheme.textTertiary,
                fontSize: 14,
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: transactions.length,
      itemBuilder: (context, index) {
        final tx = transactions[index];
        return Padding(
          padding: const EdgeInsets.only(bottom: 10),
                          child: TransactionCard(
                            txid: tx.txid,
                            amount: '${tx.amountUdya.toStringAsFixed(8)} UDAYA',
                            address: tx.address,
                            status: tx.status == TransactionStatus.confirmed
                                ? TransactionCardStatus.confirmed
                                : tx.status == TransactionStatus.pending
                                    ? TransactionCardStatus.pending
                                    : TransactionCardStatus.failed,
                            timestamp: tx.timestamp,
                            confirmations: tx.confirmations,
                            fee: '${(tx.fee / AppConstants.satoshisPerCoin).toStringAsFixed(8)}',
                            blockHeight: tx.blockHeight,
                            onTap: () => context.push('/transactions/detail', extra: tx),
                          ),
        );
      },
    );
  }
}

class _FilterOption extends StatelessWidget {
  final String label;
  final IconData icon;
  final bool isSelected;
  final VoidCallback onTap;

  const _FilterOption({required this.label, required this.icon, required this.isSelected, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
        decoration: BoxDecoration(
          color: isSelected ? UdayaTheme.secondaryColor.withValues(alpha: 0.12) : UdayaTheme.bgSurface.withValues(alpha: 0.4),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSelected ? UdayaTheme.secondaryColor.withValues(alpha: 0.3) : UdayaTheme.dividerColor,
            width: 1,
          ),
        ),
        child: Row(
          children: [
            Icon(icon, size: 18, color: isSelected ? UdayaTheme.secondaryColor : UdayaTheme.textSecondary),
            const SizedBox(width: 10),
            Text(
              label,
              style: TextStyle(
                color: isSelected ? UdayaTheme.textPrimary : UdayaTheme.textSecondary,
                fontSize: 14,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

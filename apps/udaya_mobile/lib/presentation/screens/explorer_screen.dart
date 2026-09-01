import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/block_list_item.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:shimmer/shimmer.dart';

class ExplorerScreen extends ConsumerStatefulWidget {
  const ExplorerScreen({super.key});

  @override
  ConsumerState<ExplorerScreen> createState() => _ExplorerScreenState();
}

class _ExplorerScreenState extends ConsumerState<ExplorerScreen> with SingleTickerProviderStateMixin {
  final _searchController = TextEditingController();
  bool _isSearching = false;
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _searchController.dispose();
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final blockchainState = ref.watch(blockchainProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      appBar: GlassAppBar(
        title: 'Explorer',
        actions: [
          GlassIconButton(
            icon: Icons.search_rounded,
            iconColor: UdayaTheme.textSecondary,
            onPressed: () {
              setState(() => _isSearching = !_isSearching);
            },
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () async {
          await ref.read(blockchainProvider.notifier).refresh();
        },
        child: LayoutBuilder(
          builder: (context, constraints) {
            return Column(
              children: [
                if (_isSearching) _buildSearchBar(),
                Expanded(
                  child: blockchainState.isLoading && blockchainState.blocks.isEmpty
                      ? _buildShimmerLoader()
                      : blockchainState.error != null
                          ? ErrorDisplay(
                              message: blockchainState.error!,
                              onRetry: () => ref.read(blockchainProvider.notifier).refresh(),
                            )
                          : _buildContent(blockchainState),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildShimmerLoader() {
    return Shimmer.fromColors(
      baseColor: UdayaTheme.bgCard,
      highlightColor: UdayaTheme.bgCardHover,
      child: ListView.builder(
        padding: const EdgeInsets.symmetric(vertical: 8),
        itemCount: 8,
        itemBuilder: (context, index) => Container(
          margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
          height: 72,
          decoration: BoxDecoration(
            color: UdayaTheme.bgCard,
            borderRadius: BorderRadius.circular(UdayaTheme.cardRadius),
          ),
        ),
      ),
    );
  }

  Widget _buildSearchBar() {
    return Container(
      padding: const EdgeInsets.all(16),
      child: SearchBar(
        controller: _searchController,
        hintText: 'Search by block height, txid, or address',
        hintStyle: WidgetStateProperty.all(TextStyle(color: UdayaTheme.textMuted)),
        textStyle: WidgetStateProperty.all(TextStyle(color: UdayaTheme.textPrimary)),
        backgroundColor: WidgetStateProperty.all(UdayaTheme.bgSurface),
        overlayColor: WidgetStateProperty.all(UdayaTheme.bgCard.withValues(alpha: 0.8)),
        elevation: WidgetStateProperty.all(0),
        side: WidgetStateProperty.all(BorderSide(color: UdayaTheme.borderColor, width: 1)),
        leading: Icon(Icons.search_rounded, color: UdayaTheme.textTertiary),
        trailing: [
          IconButton(
            icon: Icon(Icons.clear_rounded, color: UdayaTheme.textMuted),
            onPressed: () {
              _searchController.clear();
              setState(() => _isSearching = false);
            },
          ),
        ],
        onSubmitted: (value) => _performSearch(value),
      ),
    );
  }

  Widget _buildContent(blockchainState) {
    return DefaultTabController(
      length: 3,
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            decoration: BoxDecoration(
              color: UdayaTheme.bgDeep.withValues(alpha: 0.8),
            ),
            child: TabBar(
              controller: _tabController,
              labelColor: UdayaTheme.secondaryColor,
              unselectedLabelColor: UdayaTheme.textSecondary,
              indicatorColor: UdayaTheme.secondaryColor,
              indicatorSize: TabBarIndicatorSize.label,
              labelStyle: TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
              unselectedLabelStyle: TextStyle(fontSize: 13),
              tabs: const [
                Tab(text: 'Blocks'),
                Tab(text: 'Mempool'),
                Tab(text: 'Stats'),
              ],
            ),
          ),
          Expanded(
            child: TabBarView(
              controller: _tabController,
              children: [
                _buildBlocksList(blockchainState),
                _buildMempoolView(blockchainState),
                _buildStatsView(blockchainState),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBlocksList(blockchainState) {
    if (blockchainState.blocks.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.inbox_rounded, size: 56, color: UdayaTheme.textMuted.withValues(alpha: 0.5)),
            const SizedBox(height: 16),
            Text(
              'No blocks found',
              style: TextStyle(fontSize: 16, color: UdayaTheme.textTertiary),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: blockchainState.blocks.length,
      itemBuilder: (context, index) {
        final block = blockchainState.blocks[index];
        return BlockListItem(
          block: block,
          onTap: () => context.push('/explorer/block/${block.height}'),
        );
      },
    );
  }

  Widget _buildMempoolView(blockchainState) {
    return RefreshIndicator(
      onRefresh: () async => ref.read(blockchainProvider.notifier).refresh(),
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        physics: const AlwaysScrollableScrollPhysics(),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            GlassCard(
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
                        child: Icon(Icons.pending_rounded, color: UdayaTheme.warning, size: 18),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Mempool Status',
                              style: TextStyle(
                                fontSize: 16,
                                fontWeight: FontWeight.w600,
                                color: UdayaTheme.textPrimary,
                              ),
                            ),
                            Text(
                              'Pending transactions',
                              style: TextStyle(
                                fontSize: 13,
                                color: UdayaTheme.textTertiary,
                              ),
                            ),
                          ],
                        ),
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                        decoration: BoxDecoration(
                          color: UdayaTheme.warning.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                        ),
                        child: Text(
                          '${blockchainState.mempoolTxs.length}',
                          style: TextStyle(
                            color: UdayaTheme.warning,
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(
                        child: _StatCard(
                          label: 'Transactions',
                          value: '${blockchainState.mempoolTxs.length}',
                          color: UdayaTheme.warning,
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: _StatCard(
                          label: 'Size',
                          value: '~${(blockchainState.mempoolTxs.length * 250 / 1024).toStringAsFixed(1)} KB',
                          color: UdayaTheme.info,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            if (blockchainState.mempoolTxs.isNotEmpty) ...[
              Text(
                'Pending Transactions',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                ),
              ),
              const SizedBox(height: 10),
              ...blockchainState.mempoolTxs.take(20).map((tx) => GlassCard(
                    margin: const EdgeInsets.only(bottom: 8),
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                    child: ListTile(
                      contentPadding: EdgeInsets.zero,
                      leading: Container(
                        width: 36,
                        height: 36,
                        decoration: BoxDecoration(
                          color: UdayaTheme.warning.withValues(alpha: 0.12),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Icon(Icons.access_time_rounded, color: UdayaTheme.warning, size: 18),
                      ),
                      title: Text(
                        tx.txid.truncateMiddle,
                        style: TextStyle(
                          color: UdayaTheme.textPrimary,
                          fontFamily: 'monospace',
                          fontSize: 13,
                        ),
                      ),
                      subtitle: Text(
                        '${tx.amountUdya.toStringAsFixed(8)} UDAYA',
                        style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 12),
                      ),
                      trailing: Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
                    ),
                  )),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildStatsView(blockchainState) {
    return RefreshIndicator(
      onRefresh: () async => ref.read(blockchainProvider.notifier).refresh(),
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        physics: const AlwaysScrollableScrollPhysics(),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            GlassCard(
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
                        child: Icon(Icons.analytics_rounded, color: UdayaTheme.info, size: 18),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Network Statistics',
                              style: TextStyle(
                                fontSize: 16,
                                fontWeight: FontWeight.w600,
                                color: UdayaTheme.textPrimary,
                              ),
                            ),
                            Text(
                              'Blockchain overview',
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
                  const SizedBox(height: 20),
                  _StatCard(
                    label: 'Total Blocks',
                    value: blockchainState.blocks.isNotEmpty ? blockchainState.blocks.first.height.toString() : '0',
                    color: UdayaTheme.primaryColor,
                  ),
                  const SizedBox(height: 10),
                  _StatCard(
                    label: 'Connected Peers',
                    value: blockchainState.peers.length.toString(),
                    color: UdayaTheme.secondaryColor,
                  ),
                  const SizedBox(height: 10),
                  _StatCard(
                    label: 'Difficulty',
                    value: blockchainState.blocks.isNotEmpty ? _formatNumber(blockchainState.blocks.first.difficulty) : '0',
                    color: UdayaTheme.accentColor,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _performSearch(String query) {
    if (query.isEmpty) return;

    final height = int.tryParse(query);
    if (height != null) {
      context.push('/explorer/block/$height');
    } else if (query.length == 64) {
      context.push('/explorer/tx/$query');
    } else if (query.startsWith('U')) {
      context.push('/explorer/address/$query');
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invalid search query. Try block height, txid, or address.'),
          behavior: SnackBarBehavior.floating,
        ),
      );
    }
  }

  String _formatNumber(int value) {
    if (value >= 1000000000) return '${(value / 1000000000).toStringAsFixed(2)}B';
    if (value >= 1000000) return '${(value / 1000000).toStringAsFixed(2)}M';
    if (value >= 1000) return '${(value / 1000).toStringAsFixed(2)}K';
    return value.toString();
  }
}

class _StatCard extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _StatCard({required this.label, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: UdayaTheme.borderColor, width: 1),
      ),
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
}

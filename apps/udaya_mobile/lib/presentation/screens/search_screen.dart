import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';

class SearchScreen extends ConsumerStatefulWidget {
  final String initialQuery;

  const SearchScreen({super.key, this.initialQuery = ''});

  @override
  ConsumerState<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends ConsumerState<SearchScreen> {
  final TextEditingController _controller = TextEditingController();
  Timer? _debounceTimer;
  List<_SearchResult> _results = [];
  bool _isSearching = false;

  @override
  void initState() {
    super.initState();
    _controller.text = widget.initialQuery;
    if (widget.initialQuery.isNotEmpty) {
      _performSearch(widget.initialQuery);
    }
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    _controller.dispose();
    super.dispose();
  }

  void _onQueryChanged(String query) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 300), () {
      if (query.trim().isNotEmpty) {
        _performSearch(query.trim());
      } else {
        setState(() {
          _results = [];
          _isSearching = false;
        });
      }
    });
  }

  Future<void> _performSearch(String query) async {
    setState(() => _isSearching = true);
    final results = <_SearchResult>[];

    try {
      final walletState = ref.read(walletProvider);
      final blockchainState = ref.read(blockchainProvider);

      for (final tx in walletState.transactions) {
        if (tx.txid.contains(query) ||
            tx.address?.contains(query) == true) {
          results.add(_SearchResult(
            type: _SearchResultType.transaction,
            title: tx.txid,
            subtitle: _formatAmount(tx.amount),
            data: tx,
          ));
        }
      }

      for (final block in blockchainState.blocks) {
        if (block.hash.contains(query) ||
            block.height.toString() == query) {
          results.add(_SearchResult(
            type: _SearchResultType.block,
            title: 'Block #${block.height}',
            subtitle: block.hash,
            data: block,
          ));
        }
      }

      final addresses = await ref.read(walletRepositoryProvider).getAddresses();
      for (final addr in addresses) {
        if (addr.contains(query)) {
          results.add(_SearchResult(
            type: _SearchResultType.address,
            title: addr,
            subtitle: 'Wallet Address',
            data: addr,
          ));
        }
      }
    } catch (e) {
      debugPrint('Search error: $e');
    }

    setState(() {
      _results = results.take(20).toList();
      _isSearching = false;
    });
  }

  String _formatAmount(int satoshis) {
    final udya = satoshis / 100000000;
    return '${udya.toStringAsFixed(8)} UDAYA';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        elevation: 0,
        title: const Text(
          'Search',
          style: TextStyle(
            fontSize: 18,
            fontWeight: FontWeight.w600,
            color: UdayaTheme.textPrimary,
            letterSpacing: -0.3,
          ),
        ),
        actions: [
          if (_controller.text.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.clear_rounded, color: UdayaTheme.textSecondary),
              onPressed: () {
                _controller.clear();
                setState(() {
                  _results = [];
                });
              },
            ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: SearchBar(
              controller: _controller,
              onChanged: _onQueryChanged,
              hintText: 'Search transactions, blocks, addresses...',
              hintStyle: WidgetStateProperty.all(const TextStyle(color: UdayaTheme.textMuted)),
              textStyle: WidgetStateProperty.all(const TextStyle(color: UdayaTheme.textPrimary)),
              backgroundColor: WidgetStateProperty.all(UdayaTheme.bgSurface),
              overlayColor: WidgetStateProperty.all(UdayaTheme.bgCard.withValues(alpha: 0.8)),
              elevation: WidgetStateProperty.all(0),
              side: WidgetStateProperty.all(
                const BorderSide(color: UdayaTheme.borderColor, width: 1),
              ),
              leading: const Icon(Icons.search_rounded, color: UdayaTheme.textTertiary),
              trailing: [
                if (_controller.text.isNotEmpty)
                  IconButton(
                    icon: const Icon(Icons.close_rounded, color: UdayaTheme.textMuted),
                    onPressed: () {
                      _controller.clear();
                      setState(() => _results = []);
                    },
                  ),
              ],
            ),
          ),
          Expanded(child: _buildBody()),
        ],
      ),
    );
  }

  Widget _buildBody() {
    if (_isSearching) {
      return const Center(
        child: CircularProgressIndicator(color: UdayaTheme.primaryColor),
      );
    }

    if (_controller.text.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search_rounded, size: 48, color: UdayaTheme.textMuted.withValues(alpha: 0.5)),
            const SizedBox(height: 16),
            Text(
              'Search the Udaya Blockchain',
              style: TextStyle(fontSize: 16, color: UdayaTheme.textTertiary),
            ),
            const SizedBox(height: 8),
            Text(
              'Enter a transaction ID, block hash, or address',
              style: TextStyle(fontSize: 13, color: UdayaTheme.textMuted),
            ),
          ],
        ),
      );
    }

    if (_results.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search_off_rounded, size: 48, color: UdayaTheme.textMuted.withValues(alpha: 0.5)),
            const SizedBox(height: 16),
            Text(
              'No results found',
              style: TextStyle(fontSize: 16, color: UdayaTheme.textTertiary),
            ),
            const SizedBox(height: 8),
            Text(
              'Try a different search term',
              style: TextStyle(fontSize: 13, color: UdayaTheme.textMuted),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: _results.length,
      itemBuilder: (context, index) {
        final result = _results[index];
        return _SearchResultTile(result: result);
      },
    );
  }
}

class _SearchResultTile extends StatelessWidget {
  final _SearchResult result;

  const _SearchResultTile({required this.result});

  @override
  Widget build(BuildContext context) {
    IconData icon;
    Color color;
    switch (result.type) {
      case _SearchResultType.transaction:
        icon = Icons.receipt_long_rounded;
        color = UdayaTheme.confirmedColor;
        break;
      case _SearchResultType.block:
        icon = Icons.view_in_ar_rounded;
        color = UdayaTheme.info;
        break;
      case _SearchResultType.address:
        icon = Icons.account_balance_wallet_rounded;
        color = UdayaTheme.success;
        break;
    }

    return GlassCard(
      margin: const EdgeInsets.only(bottom: 8),
      onTap: () {
        switch (result.type) {
          case _SearchResultType.transaction:
            final tx = result.data as dynamic;
            context.push('/transactions/detail', extra: tx);
            break;
          case _SearchResultType.block:
            context.push('/explorer');
            break;
          case _SearchResultType.address:
            context.push('/wallet');
            break;
        }
      },
      child: ListTile(
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        leading: Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.15),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Icon(icon, color: color, size: 20),
        ),
        title: Text(
          result.title,
          style: const TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: UdayaTheme.textPrimary,
          ),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              result.subtitle,
              style: const TextStyle(fontSize: 12, color: UdayaTheme.textTertiary),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: color.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                result.type.name.toUpperCase(),
                style: TextStyle(
                  fontSize: 10,
                  fontWeight: FontWeight.w600,
                  color: color,
                  letterSpacing: 0.5,
                ),
              ),
            ),
          ],
        ),
        trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
      ),
    );
  }
}

enum _SearchResultType { transaction, block, address }

class _SearchResult {
  final _SearchResultType type;
  final String title;
  final String subtitle;
  final dynamic data;

  _SearchResult({
    required this.type,
    required this.title,
    required this.subtitle,
    required this.data,
  });
}

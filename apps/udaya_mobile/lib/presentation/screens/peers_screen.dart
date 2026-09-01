import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_dialog.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/peer_card.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';

class PeersScreen extends ConsumerStatefulWidget {
  const PeersScreen({super.key});

  @override
  ConsumerState<PeersScreen> createState() => _PeersScreenState();
}

class _PeersScreenState extends ConsumerState<PeersScreen> {
  final _peerAddressController = TextEditingController();

  @override
  void dispose() {
    _peerAddressController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final blockchainState = ref.watch(blockchainProvider);

    if (blockchainState.isLoading) {
      return Scaffold(
        backgroundColor: UdayaTheme.bgDeep,
        appBar: const GlassAppBar(title: 'Peers'),
        body: const LoadingIndicator(message: 'Loading peers...'),
      );
    }

    if (blockchainState.error != null) {
      return Scaffold(
        backgroundColor: UdayaTheme.bgDeep,
        appBar: const GlassAppBar(title: 'Peers'),
        body: ErrorDisplay(
          message: blockchainState.error!,
          onRetry: () => ref.read(blockchainProvider.notifier).updatePeers([]),
        ),
      );
    }

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: RefreshIndicator(
        onRefresh: () async {
          ref.read(blockchainProvider.notifier).updatePeers([]);
        },
        child: CustomScrollView(
          slivers: [
            SliverAppBar(
              pinned: true,
              floating: false,
              backgroundColor: Colors.transparent,
              elevation: 0,
              title: const Text(
                'Peers',
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.3,
                ),
              ),
              actions: [
                GlassIconButton(
                  icon: Icons.add,
                  iconColor: UdayaTheme.textSecondary,
                  onPressed: _showAddPeerDialog,
                ),
              ],
            ),
            SliverToBoxAdapter(
              child: _buildPeersList(blockchainState),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPeersList(blockchainState) {
    final peers = blockchainState.peers;

    if (peers.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 64),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.people_outline,
                size: 64,
                color: UdayaTheme.textTertiary,
              ),
              const SizedBox(height: 16),
              const Text(
                'No peers connected',
                style: TextStyle(
                  color: UdayaTheme.textSecondary,
                  fontSize: 16,
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                'Add peers manually or wait for connections',
                style: TextStyle(
                  color: UdayaTheme.textTertiary,
                  fontSize: 14,
                ),
              ),
              const SizedBox(height: 24),
              GlassButton(
                onPressed: _showAddPeerDialog,
                backgroundColor: UdayaTheme.primaryColor,
                foregroundColor: UdayaTheme.textPrimary,
                child: const Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.add, size: 18),
                    SizedBox(width: 8),
                    Text('Add Peer'),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
    }

    return Column(
      children: [
        _buildPeersSummary(peers),
        ListView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          padding: const EdgeInsets.symmetric(vertical: 8),
          itemCount: peers.length,
          itemBuilder: (context, index) {
            final peer = peers[index];
            return PeerCard(
              peerId: peer.address,
              height: peer.height,
              latency: peer.pingTimeMs,
              direction: peer.isOutbound ? 'OUTBOUND' : 'INBOUND',
              isConnected: true,
              protocolVersion: peer.userAgent,
              onTap: () => _showPeerDetails(peer),
            );
          },
        ),
      ],
    );
  }

  Widget _buildPeersSummary(List peers) {
    final inbound = peers.where((p) => !p.isOutbound).length;
    final outbound = peers.where((p) => p.isOutbound).length;

    return GlassCard(
      margin: const EdgeInsets.all(16),
      padding: const EdgeInsets.all(16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _SummaryItem(label: 'Total', value: peers.length.toString()),
          _SummaryItem(label: 'Inbound', value: inbound.toString()),
          _SummaryItem(label: 'Outbound', value: outbound.toString()),
        ],
      ),
    );
  }

  void _showAddPeerDialog() {
    GlassDialog.show(
      context: context,
      title: 'Add Peer',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            controller: _peerAddressController,
            autofocus: true,
            decoration: const InputDecoration(
              labelText: 'Peer Address',
              hintText: 'ip:port or hostname:port',
              prefixIcon: Icon(Icons.dns),
            ),
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: GlassButton(
                  isOutlined: true,
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  onPressed: () {
                    if (_peerAddressController.text.isNotEmpty) {
                      Navigator.pop(context);
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(
                          content: Text('Connecting to ${_peerAddressController.text}...'),
                          behavior: SnackBarBehavior.floating,
                        ),
                      );
                      _peerAddressController.clear();
                    }
                  },
                  backgroundColor: UdayaTheme.primaryColor,
                  foregroundColor: UdayaTheme.textPrimary,
                  child: const Text('Connect'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _showPeerDetails(peer) {
    GlassBottomSheet.show(
      context: context,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Peer Details',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 20,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _DetailRow(label: 'Address', value: peer.address),
          _DetailRow(label: 'User Agent', value: peer.userAgent),
          _DetailRow(label: 'Height', value: peer.height.toString()),
          _DetailRow(label: 'Ping', value: '${peer.pingTimeMs}ms'),
          _DetailRow(label: 'Direction', value: peer.isOutbound ? 'Outbound' : 'Inbound'),
          _DetailRow(
            label: 'Connected Since',
            value: peer.connectedSince.toIso8601String().substring(0, 19),
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: GlassButton(
                  isOutlined: true,
                  backgroundColor: UdayaTheme.error,
                  borderColor: UdayaTheme.error,
                  foregroundColor: UdayaTheme.error,
                  onPressed: () {
                    Navigator.pop(context);
                    _disconnectPeer(peer);
                  },
                  child: const Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.close, size: 18),
                      SizedBox(width: 8),
                      Text('Disconnect'),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  onPressed: () => Navigator.pop(context),
                  backgroundColor: UdayaTheme.primaryColor,
                  foregroundColor: UdayaTheme.textPrimary,
                  child: const Text('Close'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Future<void> _disconnectPeer(peer) async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Disconnect Peer',
      message: 'Are you sure you want to disconnect ${peer.address}?',
      confirmText: 'Disconnect',
      isDestructive: true,
      icon: Icons.link_off,
    );

    if (confirmed == true && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Disconnected from ${peer.address}'),
          behavior: SnackBarBehavior.floating,
        ),
      );
    }
  }
}

class _SummaryItem extends StatelessWidget {
  final String label;
  final String value;

  const _SummaryItem({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          value,
          style: const TextStyle(
            color: UdayaTheme.textPrimary,
            fontSize: 24,
            fontWeight: FontWeight.bold,
          ),
        ),
        Text(
          label,
          style: const TextStyle(
            color: UdayaTheme.textSecondary,
            fontSize: 12,
          ),
        ),
      ],
    );
  }
}

class _DetailRow extends StatelessWidget {
  final String label;
  final String value;

  const _DetailRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(color: UdayaTheme.textSecondary),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                color: UdayaTheme.textPrimary,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

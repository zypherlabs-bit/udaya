import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/network_status_card.dart' as nsc;
import 'package:udaya_mobile/presentation/widgets/design_system/metric_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';

class NodeScreen extends ConsumerStatefulWidget {
  const NodeScreen({super.key});

  @override
  ConsumerState<NodeScreen> createState() => _NodeScreenState();
}

class _NodeScreenState extends ConsumerState<NodeScreen> {
  final List<String> _logs = [
    '[INFO] Node initialized',
    '[INFO] Loading blockchain data...',
    '[INFO] Connected to 8 peers',
    '[INFO] Block height: 123456',
  ];
  bool _isNodeRunning = false;

  @override
  Widget build(BuildContext context) {
    final nodeState = ref.watch(nodeProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: nodeState.isLoading
          ? const LoadingIndicator(message: 'Loading node status...')
          : nodeState.error != null
              ? ErrorDisplay(
                  message: nodeState.error!,
                  onRetry: () => ref.read(nodeProvider.notifier).setLoading(true),
                )
              : CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: GlassAppBar(
                        title: 'Node Center',
                        backgroundColor: UdayaTheme.bgDark,
                        actions: [
                          GlassIconButton(
                            icon: Icons.refresh_rounded,
                            iconColor: UdayaTheme.textSecondary,
                            size: 36,
                            iconSize: 18,
                            onPressed: () => ref.read(nodeProvider.notifier).setLoading(true),
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
                            _buildControlCard(),
                            const SizedBox(height: 16),
                            if (nodeState.status != null)
                              nsc.NodeStatusCard(
                                status: nodeState.status!.status,
                                blockHeight: nodeState.status!.blockHeight,
                                chainwork: nodeState.status!.chainwork,
                                difficulty: nodeState.status!.difficulty,
                                peerCount: nodeState.status!.peerCount,
                                syncPercentage: nodeState.status!.syncPercentage,
                                lastBlockTime: nodeState.status!.lastBlockTime,
                                network: nodeState.status!.network,
                                cpuUsage: nodeState.status!.cpuUsage,
                                memoryUsage: nodeState.status!.memoryUsage,
                                diskUsageBytes: nodeState.status!.diskUsageBytes,
                              )
                            else
                              _buildStoppedState(),
                            const SizedBox(height: 16),
                            _buildDataDirectoryCard(),
                            const SizedBox(height: 16),
                            _buildLogViewer(),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
    );
  }

  Widget _buildControlCard() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: _isNodeRunning ? UdayaTheme.confirmedColor : UdayaTheme.textMuted,
                size: 10,
                pulse: _isNodeRunning,
              ),
              const SizedBox(width: 10),
              const Text(
                'Node Control',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.2,
                ),
              ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: GlassButton(
                  onPressed: _isNodeRunning ? null : _startNode,
                  backgroundColor: UdayaTheme.confirmedColor,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.play_arrow_rounded, size: 18),
                      const SizedBox(width: 8),
                      const Text('Start Node', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  onPressed: _isNodeRunning ? _stopNode : null,
                  isOutlined: true,
                  borderColor: UdayaTheme.error,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.stop_rounded, size: 18, color: UdayaTheme.error),
                      const SizedBox(width: 8),
                      Text('Stop Node', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: UdayaTheme.error)),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildStoppedState() {
    return GlassCard(
      padding: const EdgeInsets.all(24),
      child: Center(
        child: Column(
          children: [
            Icon(Icons.power_rounded, size: 48, color: UdayaTheme.textMuted),
            const SizedBox(height: 16),
            const Text(
              'Node is not running',
              style: TextStyle(
                color: UdayaTheme.textSecondary,
                fontSize: 15,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDataDirectoryCard() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Data Directory',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
              letterSpacing: -0.2,
            ),
          ),
          const SizedBox(height: 16),
          _InfoRow(label: 'Path', value: '/data/udaya'),
          _InfoRow(label: 'Network', value: 'mainnet'),
        ],
      ),
    );
  }

  Widget _buildLogViewer() {
    return GlassCard(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Text(
                'Node Logs',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.2,
                ),
              ),
              const Spacer(),
              GlassIconButton(
                icon: Icons.clear_all_rounded,
                iconColor: UdayaTheme.textTertiary,
                size: 32,
                iconSize: 16,
                onPressed: () => setState(() => _logs.clear()),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Container(
            height: 200,
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: UdayaTheme.bgSurface.withValues(alpha: 0.6),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: UdayaTheme.dividerColor, width: 1),
            ),
            child: ListView.builder(
              itemCount: _logs.length,
              itemBuilder: (context, index) {
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 2),
                  child: Text(
                    _logs[index],
                    style: const TextStyle(
                      color: UdayaTheme.textSecondary,
                      fontFamily: 'monospace',
                      fontSize: 12,
                      height: 1.5,
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _startNode() async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Start Node',
      message: 'This will start the Udaya node in the background. Continue?',
      confirmText: 'Start',
      icon: Icons.play_arrow_rounded,
    );

    if (confirmed == true) {
      setState(() => _isNodeRunning = true);
      _addLog('[INFO] Starting node...');
      await Future.delayed(const Duration(seconds: 1));
      _addLog('[INFO] Node started successfully');
    }
  }

  Future<void> _stopNode() async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Stop Node',
      message: 'This will stop the Udaya node. Are you sure?',
      confirmText: 'Stop',
      isDestructive: true,
      icon: Icons.stop_rounded,
    );

    if (confirmed == true) {
      _addLog('[INFO] Stopping node...');
      await Future.delayed(const Duration(seconds: 1));
      setState(() => _isNodeRunning = false);
      _addLog('[INFO] Node stopped');
    }
  }

  void _addLog(String log) {
    setState(() {
      _logs.add('${DateTime.now().toIso8601String().substring(11, 19)} $log');
    });
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;

  const _InfoRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 14)),
          Text(
            value,
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 14,
              fontWeight: FontWeight.w600,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }
}

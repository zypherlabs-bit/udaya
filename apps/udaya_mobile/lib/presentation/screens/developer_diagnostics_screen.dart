import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/status_chip.dart';

class DeveloperDiagnosticsScreen extends ConsumerWidget {
  const DeveloperDiagnosticsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: CustomScrollView(
        slivers: [
          const SliverAppBar(
            pinned: true,
            floating: false,
            backgroundColor: Colors.transparent,
            elevation: 0,
            title: Text(
              'Developer Diagnostics',
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
                  _buildFfiStatus(),
                  const SizedBox(height: 16),
                  _buildRustLibraryInfo(),
                  const SizedBox(height: 16),
                  _buildDeviceInfo(),
                  const SizedBox(height: 16),
                  _buildAppInfo(),
                  const SizedBox(height: 16),
                  _buildDebugOptions(),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFfiStatus() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'FFI Bridge Status',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _StatusRow(
            label: 'Bridge Status',
            widget: StatusChip.success('Connected'),
          ),
          _InfoRow(label: 'Library Path', value: 'libudaya_ffi.so'),
          _InfoRow(label: 'Protocol Version', value: '1'),
          _InfoRow(label: 'Last Error', value: 'None'),
        ],
      ),
    );
  }

  Widget _buildRustLibraryInfo() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Rust Library',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _InfoRow(label: 'Core Version', value: '0.1.0'),
          _InfoRow(label: 'FFI Version', value: '0.1.0'),
          _InfoRow(label: 'Build Date', value: '2024-01-15'),
          _InfoRow(label: 'Features', value: 'mining, wallet, node'),
        ],
      ),
    );
  }

  Widget _buildDeviceInfo() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Device Information',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _InfoRow(label: 'Platform', value: 'Android'),
          _InfoRow(label: 'OS Version', value: '14'),
          _InfoRow(label: 'Device Model', value: 'Pixel 8 Pro'),
          _InfoRow(label: 'Architecture', value: 'arm64-v8a'),
          _InfoRow(label: 'Available RAM', value: '12 GB'),
          _InfoRow(label: 'Available Storage', value: '256 GB'),
        ],
      ),
    );
  }

  Widget _buildAppInfo() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Application',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _InfoRow(label: 'App Version', value: AppConstants.version),
          _InfoRow(label: 'Build Mode', value: 'Debug'),
          _InfoRow(label: 'Build Number', value: '1'),
          _InfoRow(label: 'Flutter Version', value: '3.18.0'),
          _InfoRow(label: 'Dart Version', value: '3.0.0'),
        ],
      ),
    );
  }

  Widget _buildDebugOptions() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Debug Options',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          _DebugToggleRow(
            label: 'Verbose Logging',
            description: 'Enable detailed debug logging',
            value: false,
          ),
          _DebugToggleRow(
            label: 'Network Profiling',
            description: 'Track network request timings',
            value: false,
          ),
          _DebugToggleRow(
            label: 'Memory Tracking',
            description: 'Monitor memory allocations',
            value: false,
          ),
          _DebugToggleRow(
            label: 'Crash Reporting',
            description: 'Send crash reports to analytics',
            value: true,
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              isOutlined: true,
              onPressed: () {
                // Dump diagnostics action
              },
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.download, size: 18),
                  SizedBox(width: 8),
                  Text('Export Diagnostics'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
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
          Text(label, style: const TextStyle(color: UdayaTheme.textSecondary)),
          Text(
            value,
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }
}

class _StatusRow extends StatelessWidget {
  final String label;
  final Widget widget;

  const _StatusRow({required this.label, required this.widget});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: UdayaTheme.textSecondary)),
          widget,
        ],
      ),
    );
  }
}

class _DebugToggleRow extends StatefulWidget {
  final String label;
  final String description;
  final bool value;

  const _DebugToggleRow({
    required this.label,
    required this.description,
    required this.value,
  });

  @override
  State<_DebugToggleRow> createState() => _DebugToggleRowState();
}

class _DebugToggleRowState extends State<_DebugToggleRow> {
  late bool _value;

  @override
  void initState() {
    super.initState();
    _value = widget.value;
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  widget.label,
                  style: const TextStyle(color: UdayaTheme.textPrimary),
                ),
                Text(
                  widget.description,
                  style: const TextStyle(
                    color: UdayaTheme.textTertiary,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          Switch(
            value: _value,
            onChanged: (value) {
              setState(() => _value = value);
            },
            activeColor: UdayaTheme.secondaryColor,
          ),
        ],
      ),
    );
  }
}

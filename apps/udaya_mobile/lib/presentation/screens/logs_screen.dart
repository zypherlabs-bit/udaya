import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';

class LogsScreen extends ConsumerStatefulWidget {
  const LogsScreen({super.key});

  @override
  ConsumerState<LogsScreen> createState() => _LogsScreenState();
}

class _LogsScreenState extends ConsumerState<LogsScreen> {
  final List<LogEntry> _logs = [
    LogEntry(timestamp: '09:34:12', level: 'INFO', message: 'Node initialized'),
    LogEntry(timestamp: '09:34:13', level: 'INFO', message: 'Loading blockchain data...'),
    LogEntry(timestamp: '09:34:14', level: 'INFO', message: 'Connected to 8 peers'),
    LogEntry(timestamp: '09:34:15', level: 'WARN', message: 'High memory usage detected'),
    LogEntry(timestamp: '09:34:16', level: 'INFO', message: 'Block height: 123456'),
    LogEntry(timestamp: '09:34:17', level: 'ERROR', message: 'Failed to connect to peer 192.168.1.100'),
    LogEntry(timestamp: '09:34:18', level: 'DEBUG', message: 'Mempool sync complete'),
    LogEntry(timestamp: '09:34:19', level: 'INFO', message: 'New block received: 0000000000000000000123456789abcdef'),
  ];

  String _selectedLevel = 'ALL';
  String _searchQuery = '';
  bool _autoScroll = true;
  final _scrollController = ScrollController();
  final _searchController = TextEditingController();

  @override
  void dispose() {
    _scrollController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      appBar: GlassAppBar(
        title: 'Logs',
        actions: [
          GlassIconButton(
            icon: _autoScroll ? Icons.pause : Icons.play_arrow,
            iconColor: UdayaTheme.textSecondary,
            onPressed: () {
              setState(() => _autoScroll = !_autoScroll);
            },
          ),
          GlassIconButton(
            icon: Icons.file_download,
            iconColor: UdayaTheme.textSecondary,
            onPressed: _exportLogs,
          ),
          GlassIconButton(
            icon: Icons.delete_outline,
            iconColor: UdayaTheme.textSecondary,
            onPressed: _confirmClearLogs,
          ),
        ],
      ),
      body: Column(
        children: [
          _buildFilterBar(),
          _buildSearchBar(),
          Expanded(child: _buildLogsList()),
        ],
      ),
    );
  }

  Widget _buildFilterBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: ['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'].map((level) {
            final isSelected = _selectedLevel == level;
            return Padding(
              padding: const EdgeInsets.only(right: 8),
              child: FilterChip(
                label: Text(level),
                selected: isSelected,
                onSelected: (selected) {
                  setState(() => _selectedLevel = level);
                },
                backgroundColor: UdayaTheme.bgSurface,
                selectedColor: _getLevelColor(level).withValues(alpha: 0.3),
                labelStyle: TextStyle(
                  color: isSelected ? _getLevelColor(level) : UdayaTheme.textSecondary,
                ),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }

  Widget _buildSearchBar() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: TextField(
        controller: _searchController,
        decoration: InputDecoration(
          hintText: 'Search logs...',
          prefixIcon: const Icon(Icons.search),
          suffixIcon: _searchQuery.isNotEmpty
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: () {
                    _searchController.clear();
                    setState(() => _searchQuery = '');
                  },
                )
              : null,
          isDense: true,
        ),
        onChanged: (value) {
          setState(() => _searchQuery = value);
        },
      ),
    );
  }

  Widget _buildLogsList() {
    final filteredLogs = _logs.where((log) {
      final matchesLevel = _selectedLevel == 'ALL' || log.level == _selectedLevel;
      final matchesSearch = _searchQuery.isEmpty ||
          log.message.toLowerCase().contains(_searchQuery.toLowerCase());
      return matchesLevel && matchesSearch;
    }).toList();

    if (filteredLogs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.subject, size: 48, color: UdayaTheme.textTertiary),
            const SizedBox(height: 8),
            const Text(
              'No logs found',
              style: TextStyle(color: UdayaTheme.textSecondary),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: filteredLogs.length,
      itemBuilder: (context, index) {
        final log = filteredLogs[index];
        return _LogItem(
          log: log,
          onTap: () => _copyLog(log),
        );
      },
    );
  }

  Color _getLevelColor(String level) {
    switch (level) {
      case 'DEBUG':
        return UdayaTheme.textTertiary;
      case 'INFO':
        return UdayaTheme.info;
      case 'WARN':
        return UdayaTheme.warning;
      case 'ERROR':
        return UdayaTheme.error;
      default:
        return UdayaTheme.textPrimary;
    }
  }

  void _copyLog(LogEntry log) {
    Clipboard.setData(ClipboardData(text: '${log.timestamp} [${log.level}] ${log.message}'));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Log entry copied'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  void _exportLogs() {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Logs exported to downloads'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  Future<void> _confirmClearLogs() async {
    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Clear Logs',
      message: 'Are you sure you want to clear all logs?',
      confirmText: 'Clear',
      isDestructive: true,
      icon: Icons.delete_outline,
    );

    if (confirmed == true) {
      setState(() => _logs.clear());
    }
  }
}

class LogEntry {
  final String timestamp;
  final String level;
  final String message;

  LogEntry({
    required this.timestamp,
    required this.level,
    required this.message,
  });
}

class _LogItem extends StatelessWidget {
  final LogEntry log;
  final VoidCallback onTap;

  const _LogItem({required this.log, required this.onTap});

  @override
  Widget build(BuildContext context) {
    Color levelColor;
    switch (log.level) {
      case 'DEBUG':
        levelColor = UdayaTheme.textTertiary;
      case 'INFO':
        levelColor = UdayaTheme.info;
      case 'WARN':
        levelColor = UdayaTheme.warning;
      case 'ERROR':
        levelColor = UdayaTheme.error;
      default:
        levelColor = UdayaTheme.textPrimary;
    }

    return GlassCard(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      onTap: onTap,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 60,
            child: Text(
              log.timestamp,
              style: const TextStyle(
                color: UdayaTheme.textTertiary,
                fontFamily: 'monospace',
                fontSize: 11,
              ),
            ),
          ),
          Container(
            width: 50,
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: Text(
              log.level,
              style: TextStyle(
                color: levelColor,
                fontWeight: FontWeight.bold,
                fontSize: 11,
              ),
            ),
          ),
          Expanded(
            child: Text(
              log.message,
              style: const TextStyle(
                color: UdayaTheme.textSecondary,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

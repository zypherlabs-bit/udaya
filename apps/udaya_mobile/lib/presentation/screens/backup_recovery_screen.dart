import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/providers/wallet_provider.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_dialog.dart';

class BackupRecoveryScreen extends ConsumerStatefulWidget {
  const BackupRecoveryScreen({super.key});

  @override
  ConsumerState<BackupRecoveryScreen> createState() => _BackupRecoveryScreenState();
}

class _BackupRecoveryScreenState extends ConsumerState<BackupRecoveryScreen> {
  bool _isSeedRevealed = false;
  bool _isVerifying = false;
  bool _isLoadingSeed = false;
  List<String> _seedWords = [];
  final List<TextEditingController> _restoreControllers = List.generate(
    12,
    (_) => TextEditingController(),
  );

  @override
  void initState() {
    super.initState();
    _loadSeedPhrase();
  }

  Future<void> _loadSeedPhrase() async {
    setState(() => _isLoadingSeed = true);
    try {
      final repo = ref.read(walletRepositoryProvider);
      final seed = await repo.exportSeedPhrase(passphrase: '');
      setState(() {
        _seedWords = seed.split(' ');
      });
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to load seed phrase: $e'),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isLoadingSeed = false);
      }
    }
  }

  @override
  void dispose() {
    for (final controller in _restoreControllers) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
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
              'Backup & Recovery',
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
                  _buildWarningCard(),
                  const SizedBox(height: 16),
                  _buildSeedPhraseSection(),
                  const SizedBox(height: 16),
                  _buildVerifySection(),
                  const SizedBox(height: 16),
                  _buildRestoreSection(),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWarningCard() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: UdayaTheme.error.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: UdayaTheme.error.withValues(alpha: 0.3)),
      ),
      child: const Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.warning_amber_rounded, color: UdayaTheme.error),
              SizedBox(width: 8),
              Text(
                'SECURITY WARNING',
                style: TextStyle(
                  color: UdayaTheme.error,
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
            ],
          ),
          SizedBox(height: 8),
          Text(
            'Never share your seed phrase with anyone. Store it offline in a secure location. Anyone with access to your seed phrase can steal your funds.',
            style: TextStyle(color: UdayaTheme.error, fontSize: 13),
          ),
        ],
      ),
    );
  }

  Widget _buildSeedPhraseSection() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Seed Phrase',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Your 12-word seed phrase is the master key to your wallet.',
            style: TextStyle(color: UdayaTheme.textSecondary),
          ),
          const SizedBox(height: 16),
          if (_isLoadingSeed)
            const Center(child: Padding(
              padding: EdgeInsets.all(24),
              child: CircularProgressIndicator(),
            ))
          else if (_isSeedRevealed)
            _buildSeedGrid()
          else
            _buildHiddenSeed(),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: GlassButton(
                  backgroundColor: UdayaTheme.primaryColor,
                  foregroundColor: UdayaTheme.textPrimary,
                  onPressed: _isLoadingSeed || _seedWords.isEmpty ? null : () {
                    setState(() => _isSeedRevealed = !_isSeedRevealed);
                  },
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(_isSeedRevealed ? Icons.visibility_off : Icons.visibility, size: 18),
                      const SizedBox(width: 8),
                      Text(_isSeedRevealed ? 'Hide' : 'Reveal'),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  isOutlined: true,
                  onPressed: (_isSeedRevealed && _seedWords.isNotEmpty) ? _copySeed : null,
                  child: const Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.copy, size: 18),
                      SizedBox(width: 8),
                      Text('Copy'),
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

  Widget _buildSeedGrid() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: UdayaTheme.bgSurface,
        borderRadius: BorderRadius.circular(12),
      ),
      child: GridView.builder(
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
          crossAxisCount: 3,
          childAspectRatio: 2.5,
          crossAxisSpacing: 8,
          mainAxisSpacing: 8,
        ),
        itemCount: _seedWords.length,
        itemBuilder: (context, index) {
          return Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: UdayaTheme.bgCard,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: UdayaTheme.textTertiary.withValues(alpha: 0.3)),
            ),
            child: Row(
              children: [
                Text(
                  '${index + 1}.',
                  style: const TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
                ),
                const SizedBox(width: 4),
                Expanded(
                  child: Text(
                    _seedWords[index],
                    style: const TextStyle(
                      color: UdayaTheme.textPrimary,
                      fontSize: 13,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildHiddenSeed() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: UdayaTheme.bgSurface,
        borderRadius: BorderRadius.circular(12),
      ),
      child: const Column(
        children: [
          Icon(Icons.lock_outline, size: 48, color: UdayaTheme.textTertiary),
          SizedBox(height: 8),
          Text(
            'Tap "Reveal" to show your seed phrase',
            style: TextStyle(color: UdayaTheme.textSecondary),
          ),
        ],
      ),
    );
  }

  Widget _buildVerifySection() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Verify Seed Phrase',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Verify that you have securely stored your seed phrase.',
            style: TextStyle(color: UdayaTheme.textSecondary),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              isOutlined: true,
              onPressed: () {
                setState(() => _isVerifying = !_isVerifying);
              },
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.verified_user, size: 18),
                  SizedBox(width: 8),
                  Text('Start Verification'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRestoreSection() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Restore Wallet',
            style: TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Restore your wallet from a seed phrase backup.',
            style: TextStyle(color: UdayaTheme.textSecondary),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              backgroundColor: UdayaTheme.primaryColor,
              foregroundColor: UdayaTheme.textPrimary,
              onPressed: _showRestoreDialog,
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.restore, size: 18),
                  SizedBox(width: 8),
                  Text('Restore from Seed'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _copySeed() {
    final seed = _seedWords.join(' ');
    Clipboard.setData(ClipboardData(text: seed));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Seed phrase copied to clipboard'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  void _showRestoreDialog() {
    GlassDialog.show(
      context: context,
      title: 'Restore Wallet',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Enter your 12-word seed phrase to restore your wallet.',
            style: TextStyle(color: UdayaTheme.textSecondary),
          ),
          const SizedBox(height: 16),
          GridView.builder(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 3,
              childAspectRatio: 2.5,
              crossAxisSpacing: 8,
              mainAxisSpacing: 8,
            ),
            itemCount: 12,
            itemBuilder: (context, index) {
              return TextField(
                controller: _restoreControllers[index],
                decoration: InputDecoration(
                  labelText: '${index + 1}',
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(8),
                  ),
                ),
              );
            },
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
                  backgroundColor: UdayaTheme.primaryColor,
                  foregroundColor: UdayaTheme.textPrimary,
                  onPressed: () {
                    Navigator.pop(context);
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Wallet restored successfully'),
                        behavior: SnackBarBehavior.floating,
                      ),
                    );
                  },
                  child: const Text('Restore'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

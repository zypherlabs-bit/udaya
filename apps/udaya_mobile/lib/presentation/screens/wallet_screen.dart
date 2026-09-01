import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/extensions/extensions.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/metric_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:udaya_mobile/presentation/widgets/seed_phrase_dialog.dart';
import 'package:qr_flutter/qr_flutter.dart';

class WalletScreen extends ConsumerStatefulWidget {
  const WalletScreen({super.key});

  @override
  ConsumerState<WalletScreen> createState() => _WalletScreenState();
}

class _WalletScreenState extends ConsumerState<WalletScreen> {
  @override
  Widget build(BuildContext context) {
    final walletState = ref.watch(walletProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: walletState.isLoading
          ? const LoadingIndicator(message: 'Loading wallet...')
          : walletState.error != null
              ? ErrorDisplay(
                  message: walletState.error!,
                  onRetry: () => ref.read(walletProvider.notifier).refreshAll(),
                )
              : _buildBody(context, ref, walletState),
    );
  }

  Widget _buildBody(BuildContext context, WidgetRef ref, WalletState walletState) {
    final isCreated = ref.watch(isWalletCreatedProvider).value ?? false;
    if (!isCreated) {
      return _buildNoWalletState(context, ref);
    }

    return RefreshIndicator(
      onRefresh: () async => ref.read(walletProvider.notifier).refreshAll(),
      child: CustomScrollView(
        slivers: [
          SliverAppBar(
            expandedHeight: 100,
            floating: false,
            pinned: true,
            backgroundColor: Colors.transparent,
            flexibleSpace: FlexibleSpaceBar(
              background: Container(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                    colors: [
                      UdayaTheme.bgDark.withValues(alpha: 0.95),
                      UdayaTheme.bgDark.withValues(alpha: 0.0),
                    ],
                  ),
                ),
              ),
            ),
            title: Row(
              children: [
                Container(
                  width: 28,
                  height: 28,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      colors: [UdayaTheme.secondaryColor, UdayaTheme.info],
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                    ),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Icon(Icons.account_balance_wallet_rounded, color: UdayaTheme.textPrimary, size: 16),
                ),
                const SizedBox(width: 8),
                const Text(
                  'Wallet',
                  style: TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.w600,
                    color: UdayaTheme.textPrimary,
                    letterSpacing: -0.3,
                  ),
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
                  _buildBalanceCard(context, walletState),
                  const SizedBox(height: 16),
                  _buildActionButtons(context),
                  const SizedBox(height: 20),
                  _buildAccountsSection(context, walletState),
                  const SizedBox(height: 20),
                  _buildAddressesSection(context, walletState),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoWalletState(BuildContext context, WidgetRef ref) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                color: UdayaTheme.bgCard,
                borderRadius: BorderRadius.circular(24),
                border: Border.all(color: UdayaTheme.borderColor, width: 1),
              ),
              child: Icon(Icons.account_balance_wallet_outlined, size: 40, color: UdayaTheme.textMuted),
            ),
            const SizedBox(height: 24),
            const Text(
              'No Wallet Found',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.w600,
                color: UdayaTheme.textPrimary,
                letterSpacing: -0.5,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Create a new wallet or restore an existing one to get started.',
              textAlign: TextAlign.center,
              style: TextStyle(
                color: UdayaTheme.textSecondary,
                fontSize: 15,
                height: 1.6,
              ),
            ),
            const SizedBox(height: 32),
            SizedBox(
              width: double.infinity,
              height: 52,
              child: ElevatedButton.icon(
                onPressed: () => _showCreateWalletDialog(context, ref),
                icon: const Icon(Icons.add_rounded),
                label: const Text('Create New Wallet', style: TextStyle(fontSize: 15, fontWeight: FontWeight.w600)),
                style: ElevatedButton.styleFrom(
                  backgroundColor: UdayaTheme.primaryColor,
                  foregroundColor: UdayaTheme.textPrimary,
                ),
              ),
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              height: 52,
              child: OutlinedButton.icon(
                onPressed: () => _showRestoreWalletDialog(context, ref),
                icon: const Icon(Icons.restore_rounded),
                label: const Text('Restore Existing Wallet', style: TextStyle(fontSize: 15, fontWeight: FontWeight.w600)),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBalanceCard(BuildContext context, WalletState walletState) {
    final balance = walletState.balance?.totalUdya ?? 0;
    final confirmed = walletState.balance?.spendableUdya ?? 0;
    final pending = balance - confirmed;

    return GlassCard(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    colors: [UdayaTheme.primaryColor.withValues(alpha: 0.2), UdayaTheme.accentColor.withValues(alpha: 0.1)],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: UdayaTheme.primaryColor.withValues(alpha: 0.2), width: 1),
                ),
                child: Icon(Icons.account_balance_wallet_rounded, color: UdayaTheme.primaryColor, size: 20),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Total Balance',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: UdayaTheme.textTertiary,
                        letterSpacing: 0.3,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '${balance.toStringAsFixed(8)} UDAYA',
                      style: const TextStyle(
                        fontSize: 26,
                        fontWeight: FontWeight.w700,
                        color: UdayaTheme.textPrimary,
                        letterSpacing: -0.8,
                        height: 1.2,
                      ),
                    ),
                  ],
                ),
              ),
              StatusIndicator(
                color: UdayaTheme.confirmedColor,
                size: 8,
                pulse: false,
              ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: _BalanceMetric(
                  label: 'Confirmed',
                  value: '${confirmed.toStringAsFixed(8)}',
                  color: UdayaTheme.confirmedColor,
                ),
              ),
              Container(
                width: 1,
                height: 32,
                color: UdayaTheme.dividerColor,
              ),
              Expanded(
                child: _BalanceMetric(
                  label: 'Pending',
                  value: '${pending.toStringAsFixed(8)}',
                  color: pending > 0 ? UdayaTheme.syncingColor : UdayaTheme.textMuted,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: ElevatedButton.icon(
            onPressed: () => context.push('/send'),
            icon: const Icon(Icons.arrow_upward_rounded, size: 18),
            label: const Text('Send', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
            style: ElevatedButton.styleFrom(
              backgroundColor: UdayaTheme.primaryColor,
              foregroundColor: UdayaTheme.textPrimary,
              padding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: OutlinedButton.icon(
            onPressed: () => _showReceiveBottomSheet(context),
            icon: const Icon(Icons.arrow_downward_rounded, size: 18),
            label: const Text('Receive', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
            style: OutlinedButton.styleFrom(
              foregroundColor: UdayaTheme.secondaryColor,
              side: const BorderSide(color: UdayaTheme.secondaryColor, width: 1.5),
              padding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildAccountsSection(BuildContext context, WalletState walletState) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Accounts',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
            color: UdayaTheme.textPrimary,
            letterSpacing: -0.2,
          ),
        ),
        const SizedBox(height: 12),
        if (walletState.accounts.isEmpty)
          GlassCard(
            padding: const EdgeInsets.all(24),
            child: Center(
              child: Column(
                children: [
                  Icon(Icons.account_balance_wallet_outlined, size: 40, color: UdayaTheme.textMuted),
                  const SizedBox(height: 12),
                  const Text('No accounts found', style: TextStyle(color: UdayaTheme.textTertiary)),
                ],
              ),
            ),
          )
        else
          ...walletState.accounts.map((account) => GlassCard(
                margin: const EdgeInsets.only(bottom: 10),
                padding: const EdgeInsets.all(14),
                child: Row(
                  children: [
                    Container(
                      width: 36,
                      height: 36,
                      decoration: BoxDecoration(
                        color: account.isActive
                            ? UdayaTheme.secondaryColor.withValues(alpha: 0.15)
                            : UdayaTheme.bgSurface,
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Icon(
                        Icons.account_balance_wallet_rounded,
                        color: account.isActive ? UdayaTheme.secondaryColor : UdayaTheme.textMuted,
                        size: 18,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            account.name,
                            style: const TextStyle(
                              color: UdayaTheme.textPrimary,
                              fontWeight: FontWeight.w600,
                              fontSize: 14,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            account.derivationPath,
                            style: const TextStyle(color: UdayaTheme.textTertiary, fontSize: 12, fontFamily: 'monospace'),
                          ),
                        ],
                      ),
                    ),
                    if (account.isActive)
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        decoration: BoxDecoration(
                          color: UdayaTheme.secondaryColor.withValues(alpha: 0.12),
                          borderRadius: BorderRadius.circular(UdayaTheme.chipRadius),
                          border: Border.all(color: UdayaTheme.secondaryColor.withValues(alpha: 0.25), width: 1),
                        ),
                        child: const Text(
                          'ACTIVE',
                          style: TextStyle(color: UdayaTheme.secondaryColor, fontSize: 10, fontWeight: FontWeight.w700, letterSpacing: 0.5),
                        ),
                      ),
                  ],
                ),
              )),
      ],
    );
  }

  Widget _buildAddressesSection(BuildContext context, WalletState walletState) {
    final addresses = walletState.accounts.isNotEmpty ? walletState.accounts.first.externalAddresses : <String>[];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Addresses',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
            color: UdayaTheme.textPrimary,
            letterSpacing: -0.2,
          ),
        ),
        const SizedBox(height: 12),
        if (addresses.isEmpty)
          GlassCard(
            padding: const EdgeInsets.all(24),
            child: Center(
              child: Column(
                children: [
                  Icon(Icons.pin_drop_outlined, size: 40, color: UdayaTheme.textMuted),
                  const SizedBox(height: 12),
                  const Text('No addresses generated', style: TextStyle(color: UdayaTheme.textTertiary)),
                ],
              ),
            ),
          )
        else
          ...addresses.take(5).map((address) => GlassCard(
                margin: const EdgeInsets.only(bottom: 10),
                padding: const EdgeInsets.all(14),
                child: Row(
                  children: [
                    Container(
                      width: 32,
                      height: 32,
                      decoration: BoxDecoration(
                        color: UdayaTheme.info.withValues(alpha: 0.12),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Icon(Icons.pin_drop_rounded, color: UdayaTheme.info, size: 16),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Text(
                        address,
                        style: const TextStyle(
                          color: UdayaTheme.textPrimary,
                          fontFamily: 'monospace',
                          fontSize: 13,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    GlassIconButton(
                      icon: Icons.copy_rounded,
                      iconColor: UdayaTheme.textTertiary,
                      size: 32,
                      iconSize: 16,
                      onPressed: () {
                        Clipboard.setData(ClipboardData(text: address));
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('Address copied to clipboard'),
                            behavior: SnackBarBehavior.floating,
                          ),
                        );
                      },
                    ),
                  ],
                ),
              )),
      ],
    );
  }

  void _showCreateWalletDialog(BuildContext context, WidgetRef ref) {
    final nameController = TextEditingController(text: 'Main Wallet');
    final passphraseController = TextEditingController();

    GlassDialog.show(
      context: context,
      title: 'Create New Wallet',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: nameController,
            decoration: const InputDecoration(
              labelText: 'Wallet Name',
              prefixIcon: Icon(Icons.label_outline_rounded),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: passphraseController,
            obscureText: true,
            decoration: const InputDecoration(
              labelText: 'Passphrase',
              prefixIcon: Icon(Icons.lock_outline_rounded),
              hintText: 'Enter a strong passphrase',
            ),
          ),
        ],
      ),
    ).then((_) {
      if (context.mounted) {
        final name = nameController.text.trim();
        final passphrase = passphraseController.text;
        if (name.isEmpty || passphrase.isEmpty) return;

        ref.read(walletProvider.notifier).createWallet(name: name, passphrase: passphrase).then((_) {
          if (context.mounted) {
            ref.read(walletRepositoryProvider).exportSeedPhrase(passphrase: passphrase).then((seedPhrase) {
              final words = seedPhrase.split(' ');
              SeedPhraseDialog.show(context, words);
            });
          }
        });
      }
    });
  }

  void _showRestoreWalletDialog(BuildContext context, WidgetRef ref) {
    final seedController = TextEditingController();
    final passphraseController = TextEditingController();

    GlassDialog.show(
      context: context,
      title: 'Restore Wallet',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: seedController,
            maxLines: 3,
            decoration: const InputDecoration(
              labelText: 'Seed Phrase',
              hintText: 'Enter your 12 or 24 word seed phrase',
              prefixIcon: Icon(Icons.vpn_key_outlined),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: passphraseController,
            obscureText: true,
            decoration: const InputDecoration(
              labelText: 'Passphrase',
              prefixIcon: Icon(Icons.lock_outline_rounded),
            ),
          ),
        ],
      ),
    ).then((_) {
      if (context.mounted) {
        final seed = seedController.text.trim();
        final passphrase = passphraseController.text;
        if (seed.isEmpty || passphrase.isEmpty) return;
        ref.read(walletProvider.notifier).restoreWallet(seedPhrase: seed, passphrase: passphrase);
      }
    });
  }

  void _showReceiveBottomSheet(BuildContext context) {
    final walletState = ref.read(walletProvider);
    final address = walletState.accounts.isNotEmpty ? walletState.accounts.first.externalAddresses.firstOrNull ?? '' : '';

    GlassBottomSheet.show(
      context: context,
      height: 420,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'Receive UDAYA',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 20),
          if (address.isNotEmpty)
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
              ),
              child: QrImageView(
                data: address,
                version: QrVersions.auto,
                size: 180,
                backgroundColor: Colors.white,
                errorCorrectionLevel: QrErrorCorrectLevel.M,
              ),
            ),
          const SizedBox(height: 16),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: UdayaTheme.bgSurface.withValues(alpha: 0.5),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: UdayaTheme.dividerColor, width: 1),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    address,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 13,
                      color: UdayaTheme.textSecondary,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(width: 8),
                GlassIconButton(
                  icon: Icons.copy_rounded,
                  iconColor: UdayaTheme.textTertiary,
                  size: 32,
                  iconSize: 16,
                  onPressed: () {
                    Clipboard.setData(ClipboardData(text: address));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Address copied to clipboard'),
                        behavior: SnackBarBehavior.floating,
                      ),
                    );
                  },
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),
          const Text(
            'Share this address to receive UDAYA',
            style: TextStyle(
              fontSize: 13,
              color: UdayaTheme.textTertiary,
            ),
          ),
        ],
      ),
    );
  }
}

class _BalanceMetric extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _BalanceMetric({required this.label, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          value,
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: color,
            letterSpacing: -0.2,
          ),
        ),
        const SizedBox(height: 4),
        Text(
          label,
          style: const TextStyle(
            fontSize: 12,
            color: UdayaTheme.textTertiary,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/status_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:udaya_mobile/presentation/widgets/confirm_dialog.dart';
import 'package:udaya_mobile/presentation/widgets/fee_selector.dart';

class SendScreen extends ConsumerStatefulWidget {
  const SendScreen({super.key});

  @override
  ConsumerState<SendScreen> createState() => _SendScreenState();
}

class _SendScreenState extends ConsumerState<SendScreen> {
  final _formKey = GlobalKey<FormState>();
  final _addressController = TextEditingController();
  final _amountController = TextEditingController();
  final _labelController = TextEditingController();
  int _feeRate = AppConstants.defaultFeeRate;
  bool _isSending = false;

  @override
  void dispose() {
    _addressController.dispose();
    _amountController.dispose();
    _labelController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final walletState = ref.watch(walletProvider);
    final balance = walletState.balance?.spendableUdya ?? 0;

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: walletState.isLoading
          ? const LoadingIndicator(message: 'Loading wallet...')
          : walletState.error != null
              ? ErrorDisplay(message: walletState.error!)
              : CustomScrollView(
                  slivers: [
                    SliverAppBar(
                      expandedHeight: 80,
                      floating: false,
                      pinned: true,
                      backgroundColor: Colors.transparent,
                      title: const Text(
                        'Send',
                        style: TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                          color: UdayaTheme.textPrimary,
                          letterSpacing: -0.3,
                        ),
                      ),
                      leading: GlassIconButton(
                        icon: Icons.arrow_back_ios_rounded,
                        iconColor: UdayaTheme.textSecondary,
                        size: 40,
                        iconSize: 20,
                        onPressed: () => context.pop(),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
                        child: Form(
                          key: _formKey,
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              _buildBalanceCard(balance),
                              const SizedBox(height: 24),
                              _buildAddressField(),
                              const SizedBox(height: 16),
                              _buildAmountField(balance),
                              const SizedBox(height: 16),
                              _buildLabelField(),
                              const SizedBox(height: 24),
                              _buildFeeSelector(),
                              const SizedBox(height: 32),
                              _buildSendButton(balance),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
    );
  }

  Widget _buildBalanceCard(double balance) {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Row(
            children: [
              StatusIndicator(
                color: UdayaTheme.confirmedColor,
                size: 8,
                pulse: false,
              ),
              const SizedBox(width: 10),
              const Text(
                'Available Balance',
                style: TextStyle(
                  color: UdayaTheme.textSecondary,
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
          Text(
            '${balance.toStringAsFixed(8)} UDAYA',
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
              letterSpacing: -0.3,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAddressField() {
    return Semantics(
      textField: true,
      label: 'Recipient Address',
      hint: 'Enter Udaya address',
      child: TextFormField(
        controller: _addressController,
        style: const TextStyle(color: UdayaTheme.textPrimary),
        decoration: InputDecoration(
          labelText: 'Recipient Address',
          hintText: 'Enter Udaya address',
          hintStyle: const TextStyle(color: UdayaTheme.textMuted),
          prefixIcon: const Icon(Icons.account_balance_wallet_rounded, color: UdayaTheme.textTertiary, size: 20),
          suffixIcon: Semantics(
            button: true,
            label: 'Scan QR Code',
            child: GlassIconButton(
              icon: Icons.qr_code_scanner_rounded,
              iconColor: UdayaTheme.textTertiary,
              size: 36,
              iconSize: 18,
              onPressed: () {
                context.push('/qr-scanner').then((result) {
                  if (result != null && mounted) {
                    _addressController.text = result.toString();
                  }
                });
              },
            ),
          ),
        ),
        validator: (value) {
          if (value == null || value.isEmpty) {
            return 'Please enter a recipient address';
          }
          if (value.length < 26) {
            return 'Invalid address format';
          }
          return null;
        },
      ),
    );
  }

  Widget _buildAmountField(double balance) {
    return Semantics(
      textField: true,
      label: 'Amount in UDAYA',
      hint: '0.00000000',
      child: TextFormField(
        controller: _amountController,
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        style: const TextStyle(color: UdayaTheme.textPrimary),
        decoration: InputDecoration(
          labelText: 'Amount',
          hintText: '0.00000000',
          hintStyle: const TextStyle(color: UdayaTheme.textMuted),
          prefixIcon: const Icon(Icons.payments_rounded, color: UdayaTheme.textTertiary, size: 20),
          suffixText: 'UDAYA',
          suffixStyle: const TextStyle(color: UdayaTheme.textTertiary, fontSize: 13, fontWeight: FontWeight.w500),
        ),
        validator: (value) {
          if (value == null || value.isEmpty) {
            return 'Please enter an amount';
          }
          final amount = double.tryParse(value);
          if (amount == null || amount <= 0) {
            return 'Please enter a valid amount';
          }
          if (amount > balance) {
            return 'Insufficient balance';
          }
          return null;
        },
      ),
    );
  }

  Widget _buildLabelField() {
    return Semantics(
      textField: true,
      label: 'Optional label for this transaction',
      child: TextFormField(
        controller: _labelController,
        style: const TextStyle(color: UdayaTheme.textPrimary),
        decoration: InputDecoration(
          labelText: 'Label (optional)',
          hintText: 'Add a note for this transaction',
          hintStyle: const TextStyle(color: UdayaTheme.textMuted),
          prefixIcon: const Icon(Icons.label_outline_rounded, color: UdayaTheme.textTertiary, size: 20),
        ),
      ),
    );
  }

  Widget _buildFeeSelector() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Transaction Fee',
            style: TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 12),
          FeeSelector(
            selectedFeeRate: _feeRate,
            onFeeRateChanged: (rate) => setState(() => _feeRate = rate),
          ),
        ],
      ),
    );
  }

  Widget _buildSendButton(double balance) {
    return SizedBox(
      width: double.infinity,
      height: 52,
      child: GlassButton(
        onPressed: _isSending ? null : () => _handleSend(balance),
        isLoading: _isSending,
        animationDuration: const Duration(milliseconds: 200),
        child: const Text('Send Transaction', style: TextStyle(fontSize: 15, fontWeight: FontWeight.w600)),
      ),
    );
  }

  Future<void> _handleSend(double balance) async {
    if (!_formKey.currentState!.validate()) return;

    final amountDouble = double.parse(_amountController.text);
    final address = _addressController.text;
    final amountSats = (amountDouble * AppConstants.satoshisPerCoin).toInt();

    final confirmed = await ConfirmDialog.show(
      context,
      title: 'Confirm Transaction',
      message: 'You are about to send ${amountDouble.toStringAsFixed(8)} UDAYA to:\n\n$address\n\nFee rate: $_feeRate sat/vB',
      confirmText: 'Send',
      isDestructive: true,
      icon: Icons.send_rounded,
    );

    if (confirmed == true && mounted) {
      setState(() => _isSending = true);

      try {
        await ref.read(walletProvider.notifier).sendTransaction(
              address: address,
              amountSats: amountSats,
              feeRate: _feeRate,
            );
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Transaction broadcast successfully'),
              backgroundColor: UdayaTheme.confirmedColor,
              behavior: SnackBarBehavior.floating,
            ),
          );
          context.pop();
        }
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Failed to send: $e'),
              backgroundColor: UdayaTheme.error,
              behavior: SnackBarBehavior.floating,
            ),
          );
        }
      } finally {
        if (mounted) {
          setState(() => _isSending = false);
        }
      }
    }
  }
}

import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_app_bar.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_dialog.dart';
import 'package:udaya_mobile/presentation/widgets/loading_indicator.dart';
import 'package:udaya_mobile/presentation/widgets/error_widget.dart';
import 'package:qr_flutter/qr_flutter.dart';

class ReceiveScreen extends ConsumerStatefulWidget {
  const ReceiveScreen({super.key});

  @override
  ConsumerState<ReceiveScreen> createState() => _ReceiveScreenState();
}

class _ReceiveScreenState extends ConsumerState<ReceiveScreen> {
  final _amountController = TextEditingController();
  String? _selectedAddress;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final walletState = ref.read(walletProvider);
      if (walletState.accounts.isNotEmpty &&
          walletState.accounts.first.externalAddresses.isNotEmpty) {
        setState(() {
          _selectedAddress = walletState.accounts.first.externalAddresses.first;
        });
      }
    });
  }

  @override
  void dispose() {
    _amountController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final walletState = ref.watch(walletProvider);

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
                        'Receive',
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
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.center,
                          children: [
                            if (_selectedAddress != null) ...[
                              _buildQrCode(),
                              const SizedBox(height: 24),
                              _buildAddressDisplay(),
                              const SizedBox(height: 16),
                              _buildActionButtons(),
                              const SizedBox(height: 24),
                              _buildAmountRequest(),
                            ] else
                              _buildNoAddressState(),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
    );
  }

  Widget _buildQrCode() {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(20),
        boxShadow: [
          BoxShadow(
            color: UdayaTheme.primaryColor.withValues(alpha: 0.15),
            blurRadius: 24,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: QrImageView(
        data: _selectedAddress!,
        version: QrVersions.auto,
        size: 200,
        backgroundColor: Colors.white,
        errorCorrectionLevel: QrErrorCorrectLevel.M,
      ),
    );
  }

  Widget _buildAddressDisplay() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          const Text(
            'Your Address',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 13,
              fontWeight: FontWeight.w500,
              letterSpacing: 0.3,
            ),
          ),
          const SizedBox(height: 10),
          SelectableText(
            _selectedAddress!,
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontSize: 14,
              fontFamily: 'monospace',
            ),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        GlassButton(
          onPressed: _copyAddress,
          backgroundColor: UdayaTheme.primaryColor,
          foregroundColor: UdayaTheme.textPrimary,
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.copy_rounded, size: 16),
              const SizedBox(width: 8),
              const Text('Copy', style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
            ],
          ),
        ),
        const SizedBox(width: 12),
        GlassButton(
          onPressed: _shareAddress,
          isOutlined: true,
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.share_rounded, size: 16),
              const SizedBox(width: 8),
              const Text('Share', style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildAmountRequest() {
    return GlassCard(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Request Specific Amount',
            style: TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w600,
              color: UdayaTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            style: const TextStyle(color: UdayaTheme.textPrimary),
            decoration: InputDecoration(
              labelText: 'Amount (optional)',
              hintText: '0.00000000',
              hintStyle: const TextStyle(color: UdayaTheme.textMuted),
              prefixIcon: const Icon(Icons.payments_rounded, color: UdayaTheme.textTertiary, size: 20),
              suffixText: 'UDAYA',
              suffixStyle: const TextStyle(color: UdayaTheme.textTertiary, fontSize: 13, fontWeight: FontWeight.w500),
            ),
          ),
          const SizedBox(height: 12),
          SizedBox(
            width: double.infinity,
            child: GlassButton(
              onPressed: _generatePaymentRequest,
              isOutlined: true,
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.qr_code_rounded, size: 16),
                  const SizedBox(width: 8),
                  const Text('Generate Payment Request', style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoAddressState() {
    return Center(
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
          const SizedBox(height: 20),
          const Text(
            'No address available',
            style: TextStyle(
              color: UdayaTheme.textSecondary,
              fontSize: 16,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Please create a wallet first',
            style: TextStyle(
              color: UdayaTheme.textTertiary,
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }

  void _copyAddress() {
    if (_selectedAddress == null) return;
    Clipboard.setData(ClipboardData(text: _selectedAddress!));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Address copied to clipboard'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  void _shareAddress() {
    if (_selectedAddress == null) return;
    Clipboard.setData(ClipboardData(text: _selectedAddress!));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Address copied to clipboard'),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  void _generatePaymentRequest() {
    final amount = _amountController.text;
    if (amount.isNotEmpty && _selectedAddress != null) {
      final paymentRequest = 'udaya:$_selectedAddress?amount=$amount';
      GlassDialog.show(
        context: context,
        title: 'Payment Request',
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
              ),
              child: QrImageView(
                data: paymentRequest,
                version: QrVersions.auto,
                size: 180,
                backgroundColor: Colors.white,
                errorCorrectionLevel: QrErrorCorrectLevel.M,
              ),
            ),
            const SizedBox(height: 16),
            Text(
              '$amount UDAYA',
              style: const TextStyle(
                color: UdayaTheme.textPrimary,
                fontSize: 18,
                fontWeight: FontWeight.w700,
                letterSpacing: -0.5,
              ),
            ),
          ],
        ),
      );
    }
  }
}

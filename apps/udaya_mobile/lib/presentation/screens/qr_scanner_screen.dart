import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_icon_button.dart';
import 'package:share_plus/share_plus.dart';

class QrScannerScreen extends ConsumerStatefulWidget {
  const QrScannerScreen({super.key});

  @override
  ConsumerState<QrScannerScreen> createState() => _QrScannerScreenState();
}

class _QrScannerScreenState extends ConsumerState<QrScannerScreen> {
  MobileScannerController? _controller;
  bool _isProcessing = false;
  String? _scannedResult;
  bool _hasPermission = false;
  bool _isTorchOn = false;
  Timer? _debounceTimer;

  @override
  void initState() {
    super.initState();
    _requestCameraPermission();
  }

  Future<void> _requestCameraPermission() async {
    final status = await Permission.camera.request();
    if (mounted) {
      setState(() => _hasPermission = status.isGranted);
    }
    if (status.isGranted) {
      _initializeScanner();
    }
  }

  void _initializeScanner() {
    _controller = MobileScannerController(
      detectionSpeed: DetectionSpeed.noDuplicates,
      facing: CameraFacing.back,
    );
  }

  void _handleDetect(BarcodeCapture capture) {
    if (_isProcessing) return;

    final barcode = capture.barcodes.firstOrNull;
    if (barcode == null) return;

    final rawValue = barcode.rawValue;
    if (rawValue == null || rawValue.isEmpty) return;

    final isValid = _validateQrContent(rawValue);
    if (!isValid) return;

    _isProcessing = true;
    HapticFeedback.lightImpact();

    setState(() => _scannedResult = rawValue);
    _showResultBottomSheet(rawValue);
  }

  bool _validateQrContent(String content) {
    final trimmed = content.trim();
    if (trimmed.isEmpty) return false;

    if (trimmed.startsWith('udaya:')) {
      return true;
    }

    if (trimmed.startsWith('1') || trimmed.startsWith('2') || trimmed.startsWith('3')) {
      return true;
    }

    if (trimmed.length >= 26 && trimmed.length <= 90) {
      final regex = RegExp(r'^[a-zA-Z0-9]+$');
      if (regex.hasMatch(trimmed)) {
        return true;
      }
    }

    return false;
  }

  void _showResultBottomSheet(String result) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _ScanResultBottomSheet(
        result: result,
        onDismiss: () {
          _isProcessing = false;
          if (mounted) {
            setState(() => _scannedResult = null);
          }
        },
      ),
    );
  }

  void _toggleTorch() {
    if (_controller == null) return;
    setState(() {
      _isTorchOn = !_isTorchOn;
    });
    _controller!.toggleTorch();
  }

  @override
  Widget build(BuildContext context) {
    if (!_hasPermission) {
      return Scaffold(
        backgroundColor: UdayaTheme.bgDeep,
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.camera_alt_rounded, size: 64, color: UdayaTheme.textMuted.withValues(alpha: 0.5)),
                const SizedBox(height: 24),
                Text(
                  'Camera Permission Required',
                  style: TextStyle(fontSize: 20, fontWeight: FontWeight.w600, color: UdayaTheme.textPrimary),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),
                Text(
                  'Udaya needs camera access to scan QR codes',
                  style: TextStyle(fontSize: 14, color: UdayaTheme.textTertiary),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 24),
                GlassButton(
                  onPressed: _requestCameraPermission,
                  child: const Text('Grant Permission'),
                ),
                const SizedBox(height: 12),
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
              ],
            ),
          ),
        ),
      );
    }

    if (_controller == null) {
      return Scaffold(
        backgroundColor: UdayaTheme.bgDeep,
        body: const Center(
          child: CircularProgressIndicator(color: UdayaTheme.primaryColor),
        ),
      );
    }

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: Stack(
        children: [
          MobileScanner(
            controller: _controller!,
            onDetect: _handleDetect,
          ),
          _buildScanOverlay(),
          _buildTopBar(),
          _buildBottomControls(),
        ],
      ),
    );
  }

  Widget _buildScanOverlay() {
    return Center(
      child: Container(
        width: 260,
        height: 260,
        decoration: BoxDecoration(
          border: Border.all(color: UdayaTheme.primaryColor.withValues(alpha: 0.3), width: 2),
          borderRadius: BorderRadius.circular(24),
        ),
        child: Stack(
          children: [
            Positioned.fill(
              child: CustomPaint(
                painter: _ScanFramePainter(),
              ),
            ),
            Center(
              child: Container(
                width: 200,
                height: 200,
                decoration: BoxDecoration(
                  border: Border.all(color: UdayaTheme.primaryColor.withValues(alpha: 0.15), width: 1),
                  borderRadius: BorderRadius.circular(16),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTopBar() {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            GlassIconButton(
              icon: Icons.close_rounded,
              onPressed: () => Navigator.pop(context),
              size: 40,
            ),
            const Spacer(),
            Text(
              'Scan QR Code',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w600,
                color: UdayaTheme.textPrimary,
              ),
            ),
            const Spacer(),
            GlassIconButton(
              icon: _isTorchOn ? Icons.flash_on_rounded : Icons.flash_off_rounded,
              onPressed: _toggleTorch,
              size: 40,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomControls() {
    return SafeArea(
      child: Align(
        alignment: Alignment.bottomCenter,
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
            decoration: UdayaTheme.glassDecoration(
              opacity: 0.2,
              blur: 16,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'Position the QR code within the frame',
                  style: TextStyle(
                    fontSize: 14,
                    color: UdayaTheme.textSecondary,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  'Supports Udaya addresses and payment requests',
                  style: TextStyle(
                    fontSize: 12,
                    color: UdayaTheme.textTertiary,
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  @override
  void dispose() {
    _controller?.dispose();
    _debounceTimer?.cancel();
    super.dispose();
  }
}

class _ScanResultBottomSheet extends StatelessWidget {
  final String result;
  final VoidCallback onDismiss;

  const _ScanResultBottomSheet({required this.result, required this.onDismiss});

  @override
  Widget build(BuildContext context) {
    final isPaymentRequest = result.startsWith('udaya:');
    final displayValue = result.length > 50 ? '${result.substring(0, 50)}...' : result;

    return GlassBottomSheet(
      height: MediaQuery.of(context).size.height * 0.6,
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Center(
              child: Container(
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: UdayaTheme.borderColorStrong,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
            const SizedBox(height: 24),
            Row(
              children: [
                Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    color: UdayaTheme.success.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(14),
                  ),
                  child: Icon(Icons.qr_code_scanner_rounded, color: UdayaTheme.success, size: 24),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isPaymentRequest ? 'Payment Request' : 'Address Found',
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                          color: UdayaTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                        decoration: BoxDecoration(
                          color: UdayaTheme.success.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          isPaymentRequest ? 'PAYMENT REQUEST' : 'UDYA ADDRESS',
                          style: TextStyle(
                            fontSize: 10,
                            fontWeight: FontWeight.w600,
                            color: UdayaTheme.success,
                            letterSpacing: 0.5,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 20),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: UdayaTheme.bgSurface,
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: UdayaTheme.borderColor, width: 1),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    displayValue,
                    style: TextStyle(
                      fontSize: 13,
                      fontFamily: 'monospace',
                      color: UdayaTheme.textPrimary,
                    ),
                  ),
                  if (result.length > 50) ...[
                    const SizedBox(height: 8),
                    Text(
                      result,
                      style: TextStyle(
                        fontSize: 11,
                        fontFamily: 'monospace',
                        color: UdayaTheme.textTertiary,
                      ),
                    ),
                  ],
                ],
              ),
            ),
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: GlassButton(
                    onPressed: () {
                      Clipboard.setData(ClipboardData(text: result));
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('Copied to clipboard'),
                          backgroundColor: UdayaTheme.success,
                        ),
                      );
                    },
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(Icons.copy_rounded, size: 18, color: UdayaTheme.textPrimary),
                        const SizedBox(width: 8),
                        const Text('Copy'),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 12),
              Expanded(
                child: GlassButton(
                  onPressed: () {
                    Share.share(result, subject: 'Udaya Address');
                  },
                  isOutlined: true,
                  borderColor: UdayaTheme.secondaryColor,
                  foregroundColor: UdayaTheme.secondaryColor,
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.share_rounded, size: 18, color: UdayaTheme.secondaryColor),
                      const SizedBox(width: 8),
                      Text('Share', style: TextStyle(color: UdayaTheme.secondaryColor)),
                    ],
                  ),
                ),
              ),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: TextButton(
                onPressed: onDismiss,
                child: Text('Scan Again', style: TextStyle(color: UdayaTheme.textTertiary)),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ScanFramePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = UdayaTheme.primaryColor.withValues(alpha: 0.6)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round;

    final cornerLength = 24.0;
    final cornerWidth = 3.0;

    canvas.drawLine(Offset(0, cornerLength), Offset(0, 0), paint);
    canvas.drawLine(Offset(0, 0), Offset(cornerLength, 0), paint);

    canvas.drawLine(Offset(size.width - cornerLength, 0), Offset(size.width, 0), paint);
    canvas.drawLine(Offset(size.width, 0), Offset(size.width, cornerLength), paint);

    canvas.drawLine(Offset(size.width, size.height - cornerLength), Offset(size.width, size.height), paint);
    canvas.drawLine(Offset(size.width, size.height), Offset(size.width - cornerLength, size.height), paint);

    canvas.drawLine(Offset(cornerLength, size.height), Offset(0, size.height), paint);
    canvas.drawLine(Offset(0, size.height), Offset(0, size.height - cornerLength), paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

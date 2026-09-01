import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class QrDisplay extends StatelessWidget {
  final String data;
  final double size;
  final Color backgroundColor;
  final Color foregroundColor;
  final EdgeInsets padding;
  final Widget? embeddedImage;

  const QrDisplay({
    super.key,
    required this.data,
    this.size = 200,
    this.backgroundColor = Colors.white,
    this.foregroundColor = UdayaTheme.bgDeep,
    this.padding = const EdgeInsets.all(16),
    this.embeddedImage,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: padding,
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(16),
      ),
      child: QrImageView(
        data: data,
        version: QrVersions.auto,
        size: size,
        backgroundColor: backgroundColor,
        eyeStyle: QrEyeStyle(
          eyeShape: QrEyeShape.circle,
          color: UdayaTheme.bgDeep,
        ),
        dataModuleStyle: QrDataModuleStyle(
          dataModuleShape: QrDataModuleShape.circle,
          color: foregroundColor,
        ),
        errorStateBuilder: (context, error) {
          return Center(
            child: Text(
              'QR Error: ${error.toString()}',
              style: const TextStyle(color: UdayaTheme.error),
            ),
          );
        },
      ),
    );
  }
}

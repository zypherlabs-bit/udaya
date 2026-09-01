import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'glass_card.dart';
import 'glass_button.dart';

class GlassBottomSheet extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final double? height;
  final bool isDismissible;
  final bool enableDrag;
  final VoidCallback? onClosed;

  const GlassBottomSheet({
    super.key,
    required this.child,
    this.padding,
    this.height,
    this.isDismissible = true,
    this.enableDrag = true,
    this.onClosed,
  });

  static Future<T?> show<T>({
    required BuildContext context,
    required Widget child,
    EdgeInsetsGeometry? padding,
    double? height,
    bool isDismissible = true,
    bool enableDrag = true,
  }) {
    return showModalBottomSheet<T>(
      context: context,
      isDismissible: isDismissible,
      enableDrag: enableDrag,
      backgroundColor: Colors.transparent,
      builder: (context) => GlassBottomSheet(
        child: child,
        padding: padding,
        height: height,
        isDismissible: isDismissible,
        enableDrag: enableDrag,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: height,
      decoration: const BoxDecoration(
        borderRadius: BorderRadius.vertical(top: Radius.circular(UdayaTheme.panelRadius)),
      ),
      child: ClipRRect(
        borderRadius: const BorderRadius.vertical(top: Radius.circular(UdayaTheme.panelRadius)),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 8, sigmaY: 8),
          child: Container(
            padding: padding ?? const EdgeInsets.fromLTRB(20, 12, 20, 24),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
                colors: [
                  UdayaTheme.bgCard.withValues(alpha: 0.95),
                  UdayaTheme.bgDark.withValues(alpha: 0.98),
                ],
              ),
              borderRadius: const BorderRadius.vertical(top: Radius.circular(UdayaTheme.panelRadius)),
              border: const Border(top: BorderSide(color: UdayaTheme.borderColorStrong, width: 1)),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 36,
                  height: 4,
                  margin: const EdgeInsets.only(bottom: 16),
                  decoration: BoxDecoration(
                    color: UdayaTheme.textMuted.withValues(alpha: 0.4),
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                child,
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class GlassDialog extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final String? title;
  final Widget? titleWidget;
  final bool showCloseButton;
  final VoidCallback? onClose;
  final double? width;
  final double borderRadius;

  const GlassDialog({
    super.key,
    required this.child,
    this.padding,
    this.title,
    this.titleWidget,
    this.showCloseButton = true,
    this.onClose,
    this.width,
    this.borderRadius = UdayaTheme.panelRadius,
  });

  static Future<T?> show<T>({
    required BuildContext context,
    required Widget child,
    String? title,
    Widget? titleWidget,
    EdgeInsetsGeometry? padding,
    bool showCloseButton = true,
    bool barrierDismissible = true,
    double? width,
  }) {
    return showDialog<T>(
      context: context,
      barrierDismissible: barrierDismissible,
      barrierColor: Colors.black.withValues(alpha: 0.6),
      builder: (context) => GlassDialog(
        child: child,
        title: title,
        titleWidget: titleWidget,
        padding: padding,
        showCloseButton: showCloseButton,
        width: width,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: GlassPanel(
        width: width,
        padding: padding ?? const EdgeInsets.all(24),
        borderRadius: borderRadius,
        sigma: 32,
        color: UdayaTheme.bgCard,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (title != null || titleWidget != null || showCloseButton)
              Row(
                children: [
                  if (titleWidget != null)
                    Expanded(child: titleWidget!)
                  else if (title != null)
                    Expanded(
                      child: Text(
                        title!,
                        style: const TextStyle(
                          color: UdayaTheme.textPrimary,
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  if (showCloseButton)
                    GlassIconButton(
                      icon: Icons.close_rounded,
                      iconColor: UdayaTheme.textTertiary,
                      size: 32,
                      iconSize: 18,
                      onPressed: onClose ?? () => Navigator.of(context).pop(),
                    ),
                ],
              ),
            if (title != null || titleWidget != null || showCloseButton)
              const SizedBox(height: 16),
            child,
          ],
        ),
      ),
    );
  }
}

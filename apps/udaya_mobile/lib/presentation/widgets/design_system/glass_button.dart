import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class GlassButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final VoidCallback? onLongPress;
  final Color? backgroundColor;
  final Color? foregroundColor;
  final Color? borderColor;
  final double? width;
  final double? height;
  final EdgeInsetsGeometry? padding;
  final double borderRadius;
  final double borderWidth;
  final bool isLoading;
  final bool isOutlined;
  final bool isTonal;
  final Duration? animationDuration;

  const GlassButton({
    super.key,
    required this.child,
    this.onPressed,
    this.onLongPress,
    this.backgroundColor,
    this.foregroundColor,
    this.borderColor,
    this.width,
    this.height,
    this.padding,
    this.borderRadius = UdayaTheme.buttonRadius,
    this.borderWidth = 1,
    this.isLoading = false,
    this.isOutlined = false,
    this.isTonal = false,
    this.animationDuration,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveBg = backgroundColor ?? UdayaTheme.primaryColor;
    final effectiveFg = foregroundColor ?? UdayaTheme.textPrimary;
    final effectiveBorder = borderColor ?? UdayaTheme.borderColorStrong;

    Widget button = AnimatedContainer(
      duration: animationDuration ?? const Duration(milliseconds: 200),
      width: width,
      height: height ?? 48,
      padding: padding ?? const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
      decoration: BoxDecoration(
        color: isOutlined ? Colors.transparent : (isTonal ? effectiveBg.withValues(alpha: 0.15) : effectiveBg),
        borderRadius: BorderRadius.circular(borderRadius),
        border: Border.all(
          color: isOutlined ? effectiveBorder : effectiveBg.withValues(alpha: 0.3),
          width: borderWidth,
        ),
        boxShadow: isOutlined || isTonal
            ? []
            : [
                BoxShadow(
                  color: effectiveBg.withValues(alpha: 0.3),
                  blurRadius: 12,
                  offset: const Offset(0, 4),
                ),
              ],
      ),
      child: Center(
        child: isLoading
            ? SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  valueColor: AlwaysStoppedAnimation(effectiveFg),
                ),
              )
            : DefaultTextStyle(
                style: TextStyle(color: effectiveFg, fontSize: 14, fontWeight: FontWeight.w600, letterSpacing: 0.3),
                child: child,
              ),
      ),
    );

    if (onPressed != null || onLongPress != null) {
      button = Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: isLoading ? null : onPressed,
          onLongPress: isLoading ? null : onLongPress,
          borderRadius: BorderRadius.circular(borderRadius),
          child: button,
        ),
      );
    }

    return button;
  }
}

class GlassIconButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback? onPressed;
  final Color? iconColor;
  final Color? backgroundColor;
  final Color? borderColor;
  final double? size;
  final double iconSize;
  final double borderRadius;
  final double borderWidth;
  final bool isLoading;

  const GlassIconButton({
    super.key,
    required this.icon,
    this.onPressed,
    this.iconColor,
    this.backgroundColor,
    this.borderColor,
    this.size = 44,
    this.iconSize = 20,
    this.borderRadius = 12,
    this.borderWidth = 1,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveIconColor = iconColor ?? UdayaTheme.textSecondary;
    final effectiveBg = backgroundColor ?? UdayaTheme.bgSurface;
    final effectiveBorder = borderColor ?? UdayaTheme.borderColor;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: effectiveBg.withValues(alpha: 0.6),
        borderRadius: BorderRadius.circular(borderRadius),
        border: Border.all(color: effectiveBorder, width: borderWidth),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: isLoading ? null : onPressed,
          borderRadius: BorderRadius.circular(borderRadius),
          child: Center(
            child: isLoading
                ? SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation(effectiveIconColor)),
                  )
                : Icon(icon, color: effectiveIconColor, size: iconSize),
          ),
        ),
      ),
    );
  }
}

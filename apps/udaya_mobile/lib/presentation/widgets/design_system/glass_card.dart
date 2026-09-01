import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class GlassCard extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final VoidCallback? onTap;
  final double? width;
  final double? height;
  final bool blur;
  final double sigma;
  final Color? color;
  final Color? borderColor;
  final double borderWidth;
  final double borderRadius;
  final Duration? animationDuration;
  final bool animate;

  const GlassCard({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.onTap,
    this.width,
    this.height,
    this.blur = false,
    this.sigma = 8,
    this.color,
    this.borderColor,
    this.borderWidth = 1,
    this.borderRadius = UdayaTheme.cardRadius,
    this.animationDuration,
    this.animate = false,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveColor = color ?? UdayaTheme.bgCard;
    final effectiveBorderColor = borderColor ?? UdayaTheme.borderColor;

    Widget card = AnimatedContainer(
      duration: animationDuration ?? const Duration(milliseconds: 300),
      width: width,
      height: height,
      margin: margin,
      decoration: BoxDecoration(
        color: effectiveColor,
        borderRadius: BorderRadius.circular(borderRadius),
        border: Border.all(color: effectiveBorderColor, width: borderWidth),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.3),
            blurRadius: 20,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(borderRadius),
        child: BackdropFilter(
          filter: blur ? ImageFilter.blur(sigmaX: sigma, sigmaY: sigma) : ImageFilter.blur(sigmaX: 0, sigmaY: 0),
          child: Container(
            padding: padding,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  effectiveColor.withValues(alpha: 0.95),
                  effectiveColor.withValues(alpha: 0.85),
                ],
              ),
            ),
            child: child,
          ),
        ),
      ),
    );

    if (onTap != null) {
      card = InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(borderRadius),
        child: card,
      );
    }

    return card;
  }
}

class GlassPanel extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double? width;
  final double? height;
  final double sigma;
  final Color? color;
  final Color? borderColor;
  final double borderWidth;
  final double borderRadius;
  final List<BoxShadow>? shadows;
  final Gradient? gradient;

  const GlassPanel({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.width,
    this.height,
    this.sigma = 8,
    this.color,
    this.borderColor,
    this.borderWidth = 1,
    this.borderRadius = UdayaTheme.panelRadius,
    this.shadows,
    this.gradient,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveColor = color ?? UdayaTheme.bgSurface;
    final effectiveBorderColor = borderColor ?? UdayaTheme.borderColor;

    return Container(
      width: width,
      height: height,
      margin: margin,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(borderRadius),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(borderRadius),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: sigma, sigmaY: sigma),
          child: Container(
            padding: padding,
            decoration: BoxDecoration(
              gradient: gradient ?? LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  effectiveColor.withValues(alpha: 0.9),
                  effectiveColor.withValues(alpha: 0.7),
                ],
              ),
              border: Border.all(color: effectiveBorderColor, width: borderWidth),
              borderRadius: BorderRadius.circular(borderRadius),
            ),
            child: child,
          ),
        ),
      ),
    );
  }
}

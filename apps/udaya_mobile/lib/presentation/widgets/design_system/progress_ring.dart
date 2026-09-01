import 'dart:async';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class AnimatedNumber extends StatefulWidget {
  final String value;
  final TextStyle? style;
  final bool animate;
  final Duration duration;
  final Curve curve;

  const AnimatedNumber({
    super.key,
    required this.value,
    this.style,
    this.animate = false,
    this.duration = const Duration(milliseconds: 600),
    this.curve = Curves.easeOutCubic,
  });

  @override
  State<AnimatedNumber> createState() => _AnimatedNumberState();
}

class _AnimatedNumberState extends State<AnimatedNumber> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;
  String _displayValue = '';

  @override
  void initState() {
    super.initState();
    _displayValue = widget.value;
    if (widget.animate) {
      _controller = AnimationController(duration: widget.duration, vsync: this);
      _animation = Tween<double>(begin: 0, end: 1).animate(CurvedAnimation(parent: _controller, curve: widget.curve));
      _controller.forward();
    }
  }

  @override
  void didUpdateWidget(AnimatedNumber oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) {
      _displayValue = widget.value;
      if (widget.animate && _controller.isCompleted) {
        _controller.reset();
        _controller.forward();
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final style = widget.style ?? const TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: UdayaTheme.textPrimary);

    if (!widget.animate) {
      return Text(_displayValue, style: style);
    }

    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) {
        return Transform.scale(
          scale: 0.95 + (_animation.value * 0.05),
          child: Opacity(
            opacity: 0.7 + (_animation.value * 0.3),
            child: Text(_displayValue, style: style),
          ),
        );
      },
    );
  }

  @override
  void dispose() {
    if (widget.animate) {
      _controller.dispose();
    }
    super.dispose();
  }
}

class StatusIndicator extends StatelessWidget {
  final Color color;
  final double size;
  final bool pulse;
  final Duration? pulseDuration;

  const StatusIndicator({
    super.key,
    required this.color,
    this.size = 8,
    this.pulse = false,
    this.pulseDuration,
  });

  @override
  Widget build(BuildContext context) {
    if (!pulse) {
      return Container(
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: color,
          shape: BoxShape.circle,
        ),
      );
    }

    return _PulsingIndicator(
      color: color,
      size: size,
      duration: pulseDuration ?? const Duration(seconds: 2),
    );
  }
}

class _PulsingIndicator extends StatefulWidget {
  final Color color;
  final double size;
  final Duration duration;

  const _PulsingIndicator({required this.color, required this.size, required this.duration});

  @override
  State<_PulsingIndicator> createState() => _PulsingIndicatorState();
}

class _PulsingIndicatorState extends State<_PulsingIndicator> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _pulseAnimation;
  late Animation<double> _opacityAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(vsync: this, duration: widget.duration)..repeat();
    _pulseAnimation = Tween<double>(begin: 1.0, end: 1.8).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
    _opacityAnimation = Tween<double>(begin: 0.6, end: 0.0).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Stack(
          alignment: Alignment.center,
          children: [
            Container(
              width: widget.size * _pulseAnimation.value,
              height: widget.size * _pulseAnimation.value,
              decoration: BoxDecoration(
                color: widget.color.withValues(alpha: _opacityAnimation.value * 0.4),
                shape: BoxShape.circle,
              ),
            ),
            Container(
              width: widget.size,
              height: widget.size,
              decoration: BoxDecoration(
                color: widget.color,
                shape: BoxShape.circle,
                boxShadow: [
                  BoxShadow(
                    color: widget.color.withValues(alpha: 0.5),
                    blurRadius: 4,
                    spreadRadius: 1,
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}

class ProgressRing extends StatelessWidget {
  final double progress;
  final double size;
  final double strokeWidth;
  final Color? activeColor;
  final Color? backgroundColor;
  final Duration? animationDuration;
  final bool animate;

  const ProgressRing({
    super.key,
    required this.progress,
    this.size = 48,
    this.strokeWidth = 4,
    this.activeColor,
    this.backgroundColor,
    this.animationDuration,
    this.animate = false,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveActiveColor = activeColor ?? UdayaTheme.primaryColor;
    final effectiveBgColor = backgroundColor ?? UdayaTheme.borderColor;

    final clampedProgress = progress.clamp(0.0, 1.0);

    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: clampedProgress),
      duration: animationDuration ?? const Duration(milliseconds: 800),
      curve: Curves.easeOutCubic,
      builder: (context, value, child) {
        return SizedBox(
          width: size,
          height: size,
          child: Stack(
            alignment: Alignment.center,
            children: [
              CircularProgressIndicator(
                value: value,
                strokeWidth: strokeWidth,
                backgroundColor: effectiveBgColor,
                valueColor: AlwaysStoppedAnimation(effectiveActiveColor),
              ),
              Text(
                '${(value * 100).toInt()}%',
                style: TextStyle(
                  fontSize: size * 0.28,
                  fontWeight: FontWeight.w600,
                  color: UdayaTheme.textPrimary,
                  letterSpacing: -0.5,
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

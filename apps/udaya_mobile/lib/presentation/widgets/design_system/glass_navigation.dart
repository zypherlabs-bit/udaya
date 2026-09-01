import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'glass_button.dart';

class GlassAppBar extends StatelessWidget implements PreferredSizeWidget {
  final String? title;
  final Widget? titleWidget;
  final List<Widget>? actions;
  final Widget? leading;
  final bool showBackButton;
  final VoidCallback? onBackPressed;
  final Color? backgroundColor;
  final double elevation;
  final double height;

  const GlassAppBar({
    super.key,
    this.title,
    this.titleWidget,
    this.actions,
    this.leading,
    this.showBackButton = false,
    this.onBackPressed,
    this.backgroundColor,
    this.elevation = 0,
    this.height = kToolbarHeight,
  });

  @override
  Size get preferredSize => Size.fromHeight(height);

  @override
  Widget build(BuildContext context) {
    return Container(
      height: preferredSize.height,
      decoration: BoxDecoration(
        color: Colors.transparent,
        boxShadow: elevation > 0
            ? [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.15),
                  blurRadius: elevation * 4,
                  offset: Offset(0, elevation),
                ),
              ]
            : null,
      ),
      child: Stack(
        children: [
          if (backgroundColor != null)
            ClipRect(
              child: BackdropFilter(
                filter: ImageFilter.blur(sigmaX: 8, sigmaY: 8),
                child: Container(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                      colors: [
                        (backgroundColor ?? UdayaTheme.bgDark).withValues(alpha: 0.8),
                        (backgroundColor ?? UdayaTheme.bgDark).withValues(alpha: 0.6),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          SafeArea(
            bottom: false,
            child: Row(
              children: [
                if (showBackButton || leading != null)
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    child: leading ??
                        GlassIconButton(
                          icon: Icons.arrow_back_ios_rounded,
                          iconColor: UdayaTheme.textSecondary,
                          size: 40,
                          iconSize: 20,
                          onPressed: onBackPressed ?? () => Navigator.of(context).maybePop(),
                        ),
                  )
                else
                  const SizedBox(width: 8),
                Expanded(
                  child: titleWidget ??
                      Text(
                        title ?? '',
                        style: const TextStyle(
                          color: UdayaTheme.textPrimary,
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                          letterSpacing: -0.3,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                ),
                if (actions != null) ...actions!,
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class GlassNavigationBar extends StatelessWidget implements PreferredSizeWidget {
  final int selectedIndex;
  final ValueChanged<int> onDestinationSelected;
  final List<GlassNavDestination> destinations;
  final Color? backgroundColor;

  const GlassNavigationBar({
    super.key,
    required this.selectedIndex,
    required this.onDestinationSelected,
    required this.destinations,
    this.backgroundColor,
  });

  @override
  Size get preferredSize => const Size.fromHeight(64);

  @override
  Widget build(BuildContext context) {
    return Container(
      height: preferredSize.height,
      decoration: BoxDecoration(
        color: Colors.transparent,
      ),
      child: ClipRect(
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 8, sigmaY: 8),
          child: Container(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
                colors: [
                  (backgroundColor ?? UdayaTheme.bgDark).withValues(alpha: 0.9),
                  (backgroundColor ?? UdayaTheme.bgDark).withValues(alpha: 0.95),
                ],
              ),
              border: const Border(top: BorderSide(color: UdayaTheme.borderColor, width: 1)),
            ),
            child: Row(
              children: [
                for (final dest in destinations)
                  Expanded(
                    child: _NavItem(
                      destination: dest,
                      isSelected: selectedIndex == destinations.indexOf(dest),
                      onTap: () => onDestinationSelected(destinations.indexOf(dest)),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class GlassNavDestination {
  final IconData icon;
  final IconData activeIcon;
  final String label;
  final Color? activeColor;

  const GlassNavDestination({
    required this.icon,
    required this.activeIcon,
    required this.label,
    this.activeColor,
  });
}

class _NavItem extends StatelessWidget {
  final GlassNavDestination destination;
  final bool isSelected;
  final VoidCallback onTap;

  const _NavItem({
    required this.destination,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final activeColor = destination.activeColor ?? UdayaTheme.primaryColor;
    final color = isSelected ? activeColor : UdayaTheme.textMuted;
    final icon = isSelected ? destination.activeIcon : destination.icon;

    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 12),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: color, size: 22),
              const SizedBox(height: 4),
              Text(
                destination.label,
                style: TextStyle(
                  color: color,
                  fontSize: 11,
                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
                  letterSpacing: 0.2,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

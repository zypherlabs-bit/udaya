import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';

class UdayaAppBar extends ConsumerWidget implements PreferredSizeWidget {
  final String title;
  final List<Widget>? actions;
  final bool showNetworkBadge;
  final Widget? leading;
  final PreferredSizeWidget? bottom;

  const UdayaAppBar({
    super.key,
    required this.title,
    this.actions,
    this.showNetworkBadge = true,
    this.leading,
    this.bottom,
  });

  @override
  Size get preferredSize => Size.fromHeight(kToolbarHeight + (bottom?.preferredSize.height ?? 0));

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return AppBar(
      title: Text(title, style: const TextStyle(fontWeight: FontWeight.w600)),
      leading: leading,
      actions: [
        if (showNetworkBadge) ...[
          const Padding(
            padding: EdgeInsets.only(right: 8),
            child: NetworkBadge(),
          ),
        ],
        ...?actions,
      ],
      bottom: bottom,
    );
  }
}

class NetworkBadge extends ConsumerWidget {
  final double fontSize;

  const NetworkBadge({super.key, this.fontSize = 12});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final networkState = ref.watch(networkProvider);
    final isSyncing = networkState.isSyncing;
    final network = networkState.network;

    final (label, color) = isSyncing
        ? ('SYNCING', UdayaTheme.syncingColor)
        : switch (network) {
            AppNetwork.mainnet => ('MAINNET', UdayaTheme.mainnetColor),
            AppNetwork.testnet => ('TESTNET', UdayaTheme.testnetColor),
            AppNetwork.regtest => ('REGTEST', UdayaTheme.regtestColor),
          };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.5)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (isSyncing)
            SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(
                strokeWidth: 1.5,
                valueColor: AlwaysStoppedAnimation(color),
              ),
            ),
          if (isSyncing) const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              color: color,
              fontSize: fontSize,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.5,
            ),
          ),
        ],
      ),
    );
  }
}

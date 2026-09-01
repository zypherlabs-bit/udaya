import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/presentation/screens/dashboard_screen.dart';
import 'package:udaya_mobile/presentation/screens/wallet_screen.dart';
import 'package:udaya_mobile/presentation/screens/send_screen.dart';
import 'package:udaya_mobile/presentation/screens/receive_screen.dart';
import 'package:udaya_mobile/presentation/screens/transactions_screen.dart';
import 'package:udaya_mobile/presentation/screens/explorer_screen.dart';
import 'package:udaya_mobile/presentation/screens/node_screen.dart';
import 'package:udaya_mobile/presentation/screens/peers_screen.dart';
import 'package:udaya_mobile/presentation/screens/mining_screen.dart';
import 'package:udaya_mobile/presentation/screens/network_screen.dart';
import 'package:udaya_mobile/presentation/screens/testnet_screen.dart';
import 'package:udaya_mobile/presentation/screens/security_screen.dart';
import 'package:udaya_mobile/presentation/screens/settings_screen.dart';
import 'package:udaya_mobile/presentation/screens/backup_recovery_screen.dart';
import 'package:udaya_mobile/presentation/screens/logs_screen.dart';
import 'package:udaya_mobile/presentation/screens/developer_diagnostics_screen.dart';
import 'package:udaya_mobile/presentation/screens/exchange_screen.dart';
import 'package:udaya_mobile/presentation/screens/public_dashboard_screen.dart';
import 'package:udaya_mobile/presentation/screens/notification_settings_screen.dart';
import 'package:udaya_mobile/presentation/screens/decentralization_screen.dart';
import 'package:udaya_mobile/presentation/screens/transaction_detail_screen.dart';
import 'package:udaya_mobile/presentation/screens/search_screen.dart';
import 'package:udaya_mobile/presentation/screens/qr_scanner_screen.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';

final GlobalKey<NavigatorState> _rootNavigatorKey = GlobalKey<NavigatorState>();
final GlobalKey<NavigatorState> _shellNavigatorKey = GlobalKey<NavigatorState>();

final appRouter = GoRouter(
  navigatorKey: _rootNavigatorKey,
  initialLocation: '/dashboard',
  debugLogDiagnostics: true,
  routes: [
    ShellRoute(
      navigatorKey: _shellNavigatorKey,
      builder: (context, state, child) {
        return ScaffoldWithNavBar(child: child);
      },
      routes: [
        GoRoute(
          path: '/dashboard',
          name: 'dashboard',
          pageBuilder: (context, state) => const NoTransitionPage(
            child: DashboardScreen(),
          ),
        ),
        GoRoute(
          path: '/wallet',
          name: 'wallet',
          pageBuilder: (context, state) => const NoTransitionPage(
            child: WalletScreen(),
          ),
        ),
        GoRoute(
          path: '/explorer',
          name: 'explorer',
          pageBuilder: (context, state) => const NoTransitionPage(
            child: ExplorerScreen(),
          ),
        ),
        GoRoute(
          path: '/transactions',
          name: 'transactions',
          pageBuilder: (context, state) => const NoTransitionPage(
            child: TransactionsScreen(),
          ),
        ),
        GoRoute(
          path: '/settings',
          name: 'settings',
          pageBuilder: (context, state) => const NoTransitionPage(
            child: SettingsScreen(),
          ),
        ),
      ],
    ),
    GoRoute(
      path: '/send',
      name: 'send',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const SendScreen(),
    ),
    GoRoute(
      path: '/receive',
      name: 'receive',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const ReceiveScreen(),
    ),
    GoRoute(
      path: '/node',
      name: 'node',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const NodeScreen(),
    ),
    GoRoute(
      path: '/peers',
      name: 'peers',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const PeersScreen(),
    ),
    GoRoute(
      path: '/mining',
      name: 'mining',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const MiningScreen(),
    ),
    GoRoute(
      path: '/network',
      name: 'network',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const NetworkScreen(),
    ),
    GoRoute(
      path: '/testnet',
      name: 'testnet',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const TestnetScreen(),
    ),
    GoRoute(
      path: '/security',
      name: 'security',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const SecurityScreen(),
    ),
    GoRoute(
      path: '/backup-recovery',
      name: 'backup-recovery',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const BackupRecoveryScreen(),
    ),
    GoRoute(
      path: '/logs',
      name: 'logs',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const LogsScreen(),
    ),
    GoRoute(
      path: '/diagnostics',
      name: 'diagnostics',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const DeveloperDiagnosticsScreen(),
    ),
    GoRoute(
      path: '/qr-scanner',
      name: 'qr-scanner',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const QrScannerScreen(),
    ),
    GoRoute(
      path: '/exchange',
      name: 'exchange',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const ExchangeScreen(),
    ),
    GoRoute(
      path: '/public-dashboard',
      name: 'public-dashboard',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const PublicDashboardScreen(),
    ),
    GoRoute(
      path: '/notifications',
      name: 'notifications',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const NotificationSettingsScreen(),
    ),
    GoRoute(
      path: '/transactions/detail',
      name: 'transaction-detail',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) {
        final tx = state.extra as TransactionInfo?;
        if (tx == null) return const SizedBox.shrink();
        return TransactionDetailScreen(tx: tx);
      },
    ),
    GoRoute(
      path: '/decentralization',
      name: 'decentralization',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) => const DecentralizationScreen(),
    ),
    GoRoute(
      path: '/search',
      name: 'search',
      parentNavigatorKey: _rootNavigatorKey,
      builder: (context, state) {
        final query = state.uri.queryParameters['q'] ?? '';
        return SearchScreen(initialQuery: query);
      },
    ),
  ],
);

final appRouterProvider = Provider<GoRouter>((ref) => appRouter);

class ScaffoldWithNavBar extends StatefulWidget {
  final Widget child;

  const ScaffoldWithNavBar({super.key, required this.child});

  @override
  State<ScaffoldWithNavBar> createState() => _ScaffoldWithNavBarState();
}

class _ScaffoldWithNavBarState extends State<ScaffoldWithNavBar> {
  int _currentIndex = 0;

  static const List<_NavDestination> _destinations = [
    _NavDestination(
      icon: Icons.dashboard_outlined,
      selectedIcon: Icons.dashboard,
      label: 'Dashboard',
      route: '/dashboard',
    ),
    _NavDestination(
      icon: Icons.account_balance_wallet_outlined,
      selectedIcon: Icons.account_balance_wallet,
      label: 'Wallet',
      route: '/wallet',
    ),
    _NavDestination(
      icon: Icons.explore_outlined,
      selectedIcon: Icons.explore,
      label: 'Explorer',
      route: '/explorer',
    ),
    _NavDestination(
      icon: Icons.receipt_long_outlined,
      selectedIcon: Icons.receipt_long,
      label: 'Transactions',
      route: '/transactions',
    ),
    _NavDestination(
      icon: Icons.settings_outlined,
      selectedIcon: Icons.settings,
      label: 'Settings',
      route: '/settings',
    ),
  ];

  @override
  Widget build(BuildContext context) {
    final location = GoRouterState.of(context).uri.toString();
    _currentIndex = _destinations.indexWhere((d) => location.startsWith(d.route));
    if (_currentIndex == -1) _currentIndex = 0;

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.only(bottom: 80),
          child: widget.child,
        ),
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) {
          setState(() => _currentIndex = index);
          context.go(_destinations[index].route);
        },
        destinations: _destinations
            .map((d) => NavigationDestination(
                  icon: Icon(d.icon),
                  selectedIcon: Icon(d.selectedIcon),
                  label: d.label,
                ))
            .toList(),
      ),
    );
  }
}

class _NavDestination {
  final IconData icon;
  final IconData selectedIcon;
  final String label;
  final String route;

  const _NavDestination({
    required this.icon,
    required this.selectedIcon,
    required this.label,
    required this.route,
  });
}

class _QrScannerPlaceholder extends StatelessWidget {
  const _QrScannerPlaceholder();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('QR Scanner'),
        backgroundColor: const Color(0xFF111638),
      ),
      body: const Center(
        child: Text(
          'QR Scanner Placeholder\n(Camera permission required)',
          textAlign: TextAlign.center,
        ),
      ),
    );
  }
}

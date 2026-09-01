import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter/services.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/routes/app_router.dart';
import 'package:udaya_mobile/presentation/providers/theme_provider.dart';
import 'package:udaya_mobile/core/di/service_locator.dart';
import 'package:udaya_mobile/l10n/app_localizations.dart';
import 'package:udaya_mobile/presentation/widgets/app_lock_wrapper.dart';
import 'package:udaya_mobile/presentation/providers/app_lock_provider.dart';
import 'package:udaya_mobile/services/crash_reporting_service.dart';
import 'package:udaya_mobile/services/notification_service.dart';
import 'package:udaya_mobile/services/app_lifecycle_manager.dart';
import 'package:udaya_mobile/services/monitoring_service.dart';
import 'package:udaya_mobile/services/node_service.dart';
import 'package:udaya_mobile/services/secure_storage_service.dart';
import 'package:connectivity_plus/connectivity_plus.dart';

const platform = MethodChannel('udaya_mobile/security');

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  runApp(createProviderScope(prefs: prefs, child: const UdayaApp()));
}

class UdayaApp extends StatefulWidget {
  const UdayaApp({super.key});

  @override
  State<UdayaApp> createState() => _UdayaAppState();
}

class _UdayaAppState extends State<UdayaApp> with WidgetsBindingObserver {
  AppLifecycleManager? _lifecycleManager;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _enableScreenshotProtection();
    _initializeServices();
  }

  Future<void> _initializeServices() async {
    try {
      await CrashReportingService.initialize();
    } catch (e) {
      debugPrint('Crash reporting init failed: $e');
    }

    try {
      final notificationService = NotificationService();
      await notificationService.initialize();
    } catch (e) {
      debugPrint('Notification service init failed: $e');
    }

    final nodeService = NodeService();
    _lifecycleManager = AppLifecycleManager(
      MonitoringService(nodeService, SecureStorageService()),
      Connectivity(),
    );

    try {
      await _lifecycleManager!.initialize();
    } catch (e) {
      debugPrint('App lifecycle manager init failed: $e');
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      try {
        if (mounted) {
          final container = ProviderScope.containerOf(context, listen: false);
          container.read(appLockProvider.notifier).onAppResumed();
        }
      } catch (e) {
        // App lock notifier may not be available
      }
    }
  }

  Future<void> _enableScreenshotProtection() async {
    try {
      await platform.invokeMethod('enableScreenshotProtection');
    } on PlatformException catch (e) {
      debugPrint('Failed to enable screenshot protection: ${e.message}');
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _lifecycleManager?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Consumer(
      builder: (context, ref, child) {
        final themeMode = ref.watch(themeModeProvider);

        return MaterialApp.router(
          title: 'Udaya',
          debugShowCheckedModeBanner: false,
          theme: UdayaTheme.lightTheme,
          darkTheme: UdayaTheme.darkTheme,
          themeMode: themeMode,
          routerConfig: ref.watch(appRouterProvider),
          localizationsDelegates: const [
            AppLocalizationsDelegate(),
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          supportedLocales: const [
            Locale('en', ''),
          ],
          builder: (context, child) {
            return AppLockWrapper(child: child!);
          },
        );
      },
    );
  }
}

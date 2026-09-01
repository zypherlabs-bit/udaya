import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/presentation/providers/app_providers.dart';
import 'package:udaya_mobile/presentation/providers/theme_provider.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_card.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_button.dart';
import 'package:udaya_mobile/presentation/widgets/design_system/glass_bottom_sheet.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  String _selectedCurrency = 'USD';
  String _selectedLanguage = 'English';
  bool _isDarkMode = true;
  String _rpcEndpoint = '';
  String _dataUsage = 'wifi_only';
  String _addressType = 'native';
  bool _backupReminder = true;
  bool _hideBalance = false;
  bool _privacyMode = false;
  bool _clipboardClear = true;
  bool _screenshotProtection = false;
  bool _dynamicColor = false;
  bool _biometricEnabled = false;
  bool _autoLockEnabled = true;
  bool _developerOptions = false;
  String _logLevel = 'info';
  bool _notificationsMasterEnabled = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final prefs = ref.read(sharedPrefsSourceProvider);
    setState(() {
      _selectedCurrency = prefs.getString('selected_currency') ?? 'USD';
      _selectedLanguage = prefs.getString('selected_language') ?? 'English';
      _isDarkMode = prefs.getThemeMode() == 'dark';
      _rpcEndpoint = prefs.getString('rpc_endpoint') ?? '';
      _dataUsage = prefs.getString('data_usage') ?? 'wifi_only';
      _addressType = prefs.getString('address_type') ?? 'native';
      _backupReminder = prefs.getBool('backup_reminder', defaultValue: true);
      _hideBalance = prefs.getBool('hide_balance', defaultValue: false);
      _privacyMode = prefs.getBool('privacy_mode', defaultValue: false);
      _clipboardClear = prefs.getBool('clipboard_clear', defaultValue: true);
      _screenshotProtection = prefs.getBool('screenshot_protection', defaultValue: false);
      _dynamicColor = prefs.getBool('dynamic_color', defaultValue: false);
      _biometricEnabled = prefs.getBool('biometric_enabled', defaultValue: false);
      _autoLockEnabled = prefs.getBool('auto_lock_enabled', defaultValue: true);
      _developerOptions = prefs.getBool('developer_options', defaultValue: false);
      _logLevel = prefs.getString('log_level') ?? 'info';
      _notificationsMasterEnabled = prefs.getBool('notifications_master', defaultValue: true);
    });
  }

  Future<void> _saveSettings() async {
    final prefs = ref.read(sharedPrefsSourceProvider);
    await prefs.setString('selected_currency', _selectedCurrency);
    await prefs.setString('selected_language', _selectedLanguage);
    await prefs.setThemeMode(_isDarkMode ? 'dark' : 'light');
    await prefs.setString('rpc_endpoint', _rpcEndpoint);
    await prefs.setString('data_usage', _dataUsage);
    await prefs.setString('address_type', _addressType);
    await prefs.setBool('backup_reminder', _backupReminder);
    await prefs.setBool('hide_balance', _hideBalance);
    await prefs.setBool('privacy_mode', _privacyMode);
    await prefs.setBool('clipboard_clear', _clipboardClear);
    await prefs.setBool('screenshot_protection', _screenshotProtection);
    await prefs.setBool('dynamic_color', _dynamicColor);
    await prefs.setBool('biometric_enabled', _biometricEnabled);
    await prefs.setBool('auto_lock_enabled', _autoLockEnabled);
    await prefs.setBool('developer_options', _developerOptions);
    await prefs.setString('log_level', _logLevel);
    await prefs.setBool('notifications_master', _notificationsMasterEnabled);
  }

  @override
  Widget build(BuildContext context) {
    final networkState = ref.watch(networkProvider);

    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      body: CustomScrollView(
        slivers: [
          SliverAppBar(
            pinned: true,
            floating: false,
            backgroundColor: Colors.transparent,
            elevation: 0,
            title: Row(
              children: [
                Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      colors: [UdayaTheme.primaryColor, UdayaTheme.accentColor],
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                    ),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(Icons.settings_rounded, color: UdayaTheme.textPrimary, size: 18),
                ),
                const SizedBox(width: 12),
                const Text(
                  'Settings',
                  style: TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.w600,
                    color: UdayaTheme.textPrimary,
                    letterSpacing: -0.3,
                  ),
                ),
              ],
            ),
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _SettingsSection(
                    title: 'Appearance',
                    subtitle: 'Theme, colors and display preferences',
                    icon: Icons.palette_rounded,
                    iconColor: UdayaTheme.primaryColor,
                    children: [
                      _buildAppearanceSection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Network',
                    subtitle: 'Blockchain network configuration',
                    icon: Icons.hub_rounded,
                    iconColor: UdayaTheme.info,
                    children: [
                      _buildNetworkSection(networkState),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Wallet',
                    subtitle: 'Address and currency preferences',
                    icon: Icons.account_balance_wallet_rounded,
                    iconColor: UdayaTheme.secondaryColor,
                    children: [
                      _buildWalletSection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Notifications',
                    subtitle: 'Manage notification preferences',
                    icon: Icons.notifications_rounded,
                    iconColor: UdayaTheme.warning,
                    children: [
                      _buildNotificationsSection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Privacy',
                    subtitle: 'Balance visibility and data protection',
                    icon: Icons.visibility_off_rounded,
                    iconColor: UdayaTheme.accentColor,
                    children: [
                      _buildPrivacySection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Security',
                    subtitle: 'Biometrics, lock and protection',
                    icon: Icons.security_rounded,
                    iconColor: UdayaTheme.success,
                    children: [
                      _buildSecuritySection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'Advanced',
                    subtitle: 'Developer options and diagnostics',
                    icon: Icons.code_rounded,
                    iconColor: UdayaTheme.textTertiary,
                    children: [
                      _buildAdvancedSection(),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SettingsSection(
                    title: 'About',
                    subtitle: 'Version, build and legal',
                    icon: Icons.info_rounded,
                    iconColor: UdayaTheme.textSecondary,
                    children: [
                      _buildAboutSection(),
                    ],
                  ),
                  const SizedBox(height: 32),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAppearanceSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SettingsTile(
          title: 'Dark Mode',
          subtitle: _isDarkMode ? 'Dark theme active' : 'Light theme active',
          icon: _isDarkMode ? Icons.dark_mode_rounded : Icons.light_mode_rounded,
          trailing: Switch(
            value: _isDarkMode,
            onChanged: (value) {
              setState(() => _isDarkMode = value);
              ref.read(themeModeProvider.notifier).setThemeMode(value ? ThemeMode.dark : ThemeMode.light);
              _saveSettings();
            },
            activeThumbColor: UdayaTheme.secondaryColor,
          ),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'Accent Color',
          subtitle: 'Primary interface color',
          icon: Icons.color_lens_rounded,
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildColorDot(UdayaTheme.primaryColor),
              const SizedBox(width: 8),
              _buildColorDot(UdayaTheme.secondaryColor),
              const SizedBox(width: 8),
              _buildColorDot(UdayaTheme.accentColor),
            ],
          ),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        SwitchListTile(
          title: Text('Dynamic Color', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Use system accent color', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _dynamicColor,
          onChanged: (value) {
            setState(() => _dynamicColor = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
      ],
    );
  }

  Widget _buildColorDot(Color color) {
    return Container(
      width: 20,
      height: 20,
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
        border: Border.all(color: UdayaTheme.borderColorStrong, width: 1.5),
      ),
    );
  }

  Widget _buildNetworkSection(NetworkState networkState) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ...AppNetwork.values.map((network) {
          final isSelected = networkState.network == network;
          return RadioListTile<AppNetwork>(
            title: Text(
              _networkLabel(network),
              style: TextStyle(
                color: UdayaTheme.textPrimary,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
                fontSize: 15,
              ),
            ),
            subtitle: Text(
              _networkDescription(network),
              style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12),
            ),
            value: network,
            groupValue: networkState.network,
            onChanged: (value) {
              if (value != null) {
                ref.read(networkProvider.notifier).setNetwork(value);
                _saveSettings();
              }
            },
            activeColor: _networkColor(network),
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            dense: true,
          );
        }).toList(),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'RPC Endpoint',
          subtitle: _rpcEndpoint.isNotEmpty ? _rpcEndpoint : 'Default endpoint',
          icon: Icons.dns_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => _showRpcEndpointDialog(),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _buildDataUsageSection(),
      ],
    );
  }

  Widget _buildDataUsageSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(
            'Data Usage',
            style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 12, fontWeight: FontWeight.w500, letterSpacing: 0.3),
          ),
        ),
        RadioListTile<String>(
          title: Text('WiFi Only', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
          subtitle: Text('Sync only on WiFi', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12)),
          value: 'wifi_only',
          groupValue: _dataUsage,
          onChanged: (value) {
            if (value != null) {
              setState(() => _dataUsage = value);
              _saveSettings();
            }
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 2),
          dense: true,
        ),
        RadioListTile<String>(
          title: Text('WiFi and Mobile', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
          subtitle: Text('Sync on any connection', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12)),
          value: 'mobile_data',
          groupValue: _dataUsage,
          onChanged: (value) {
            if (value != null) {
              setState(() => _dataUsage = value);
              _saveSettings();
            }
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 2),
          dense: true,
        ),
        RadioListTile<String>(
          title: Text('Background Sync', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
          subtitle: Text('Sync in background', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 12)),
          value: 'background_sync',
          groupValue: _dataUsage,
          onChanged: (value) {
            if (value != null) {
              setState(() => _dataUsage = value);
              _saveSettings();
            }
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 2),
          dense: true,
        ),
      ],
    );
  }

  Widget _buildWalletSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SettingsTile(
          title: 'Default Address Type',
          subtitle: _addressTypeLabel(_addressType),
          icon: Icons.key_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => _showAddressTypeDialog(),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'Display Currency',
          subtitle: _selectedCurrency,
          icon: Icons.attach_money_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => _showCurrencyDialog(),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        SwitchListTile(
          title: Text('Backup Reminder', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Show backup reminders periodically', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _backupReminder,
          onChanged: (value) {
            setState(() => _backupReminder = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
      ],
    );
  }

  Widget _buildNotificationsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SwitchListTile(
          title: Text('Enable Notifications', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text(_notificationsMasterEnabled ? 'Notifications are enabled' : 'Notifications are disabled', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _notificationsMasterEnabled,
          onChanged: (value) {
            setState(() => _notificationsMasterEnabled = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'Notification Settings',
          subtitle: 'Transaction, node and security alerts',
          icon: Icons.tune_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => context.push('/notifications'),
        ),
      ],
    );
  }

  Widget _buildPrivacySection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SwitchListTile(
          title: Text('Hide Balance', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Mask wallet balance by default', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _hideBalance,
          onChanged: (value) {
            setState(() => _hideBalance = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        SwitchListTile(
          title: Text('Privacy Mode', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Enhanced privacy for transactions', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _privacyMode,
          onChanged: (value) {
            setState(() => _privacyMode = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        SwitchListTile(
          title: Text('Auto-clear Clipboard', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Clear clipboard after 60 seconds', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _clipboardClear,
          onChanged: (value) {
            setState(() => _clipboardClear = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        SwitchListTile(
          title: Text('Screenshot Protection', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Prevent screenshots of sensitive data', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _screenshotProtection,
          onChanged: (value) {
            setState(() => _screenshotProtection = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
      ],
    );
  }

  Widget _buildSecuritySection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SettingsTile(
          title: 'Biometric Authentication',
          subtitle: _biometricEnabled ? 'Enabled' : 'Not configured',
          icon: Icons.fingerprint_rounded,
          trailing: Switch(
            value: _biometricEnabled,
            onChanged: (value) async {
              setState(() => _biometricEnabled = value);
              await _saveSettings();
            },
            activeThumbColor: UdayaTheme.success,
          ),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'App Lock Timeout',
          subtitle: _autoLockEnabled ? 'Auto-lock enabled' : 'Auto-lock disabled',
          icon: Icons.lock_clock_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => context.push('/security'),
        ),
        const Divider(height: 1, indent: 56, endIndent: 16),
        _SettingsTile(
          title: 'Security Center',
          subtitle: 'Audit, fuzz tests and diagnostics',
          icon: Icons.shield_rounded,
          trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
          onTap: () => context.push('/security'),
        ),
      ],
    );
  }

  Widget _buildAdvancedSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SwitchListTile(
          title: Text('Developer Options', style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 15, fontWeight: FontWeight.w500)),
          subtitle: Text('Enable debugging and diagnostics', style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 13)),
          value: _developerOptions,
          onChanged: (value) {
            setState(() => _developerOptions = value);
            _saveSettings();
          },
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          dense: true,
        ),
        if (_developerOptions) ...[
          const Divider(height: 1, indent: 56, endIndent: 16),
          _SettingsTile(
            title: 'Log Level',
            subtitle: _logLevel.toUpperCase(),
            icon: Icons.bug_report_rounded,
            trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
            onTap: () => _showLogLevelDialog(),
          ),
          const Divider(height: 1, indent: 56, endIndent: 16),
          _SettingsTile(
            title: 'Clear Cache',
            subtitle: 'Free up storage space',
            icon: Icons.cleaning_services_rounded,
            trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
            onTap: () => _confirmClearCache(),
          ),
          const Divider(height: 1, indent: 56, endIndent: 16),
          _SettingsTile(
            title: 'Reset Settings',
            subtitle: 'Restore all settings to defaults',
            icon: Icons.restore_rounded,
            iconColor: UdayaTheme.error,
            titleColor: UdayaTheme.error,
            trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: UdayaTheme.textMuted),
            onTap: () => _confirmResetSettings(),
          ),
        ],
      ],
    );
  }

  Widget _buildAboutSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _AboutRow(label: 'Version', value: '1.0.0-alpha.1'),
        _AboutRow(label: 'Build', value: '2024.01.15'),
        _AboutRow(label: 'Rust Core', value: '0.1.0'),
        const Divider(height: 16, indent: 0, endIndent: 0),
        Row(
          children: [
            Expanded(
              child: GlassButton(
                isOutlined: true,
                onPressed: () {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Checking for updates...'),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                },
                child: const Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.system_update, size: 18),
                    SizedBox(width: 8),
                    Text('Check Updates'),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: GlassButton(
                isOutlined: true,
                onPressed: () => showLicensePage(context: context),
                child: const Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.description, size: 18),
                    SizedBox(width: 8),
                    Text('Licenses'),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: GlassButton(
                isOutlined: true,
                onPressed: () {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Opening documentation...'),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                },
                child: const Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.help_outline_rounded, size: 18),
                    SizedBox(width: 8),
                    Text('Docs'),
                  ],
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }

  String _networkLabel(AppNetwork network) {
    switch (network) {
      case AppNetwork.mainnet:
        return 'Mainnet';
      case AppNetwork.testnet:
        return 'Testnet';
      case AppNetwork.regtest:
        return 'Regtest';
    }
  }

  String _networkDescription(AppNetwork network) {
    switch (network) {
      case AppNetwork.mainnet:
        return 'Production Bitcoin network';
      case AppNetwork.testnet:
        return 'Testing network with test coins';
      case AppNetwork.regtest:
        return 'Local regression test network';
    }
  }

  Color _networkColor(AppNetwork network) {
    switch (network) {
      case AppNetwork.mainnet:
        return UdayaTheme.mainnetColor;
      case AppNetwork.testnet:
        return UdayaTheme.testnetColor;
      case AppNetwork.regtest:
        return UdayaTheme.regtestColor;
    }
  }

  String _addressTypeLabel(String type) {
    switch (type) {
      case 'native':
        return 'Native SegWit (bc1)';
      case 'segwit':
        return 'SegWit (P2SH)';
      case 'legacy':
        return 'Legacy (P2PKH)';
      default:
        return type;
    }
  }

  void _showRpcEndpointDialog() {
    final controller = TextEditingController(text: _rpcEndpoint);
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: UdayaTheme.bgCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(UdayaTheme.dialogRadius),
          side: const BorderSide(color: UdayaTheme.borderColorStrong, width: 1),
        ),
        title: const Text('RPC Endpoint', style: TextStyle(color: UdayaTheme.textPrimary, fontWeight: FontWeight.w600)),
        content: TextField(
          controller: controller,
          style: const TextStyle(color: UdayaTheme.textPrimary),
          decoration: InputDecoration(
            hintText: 'http://127.0.0.1:8332',
            hintStyle: TextStyle(color: UdayaTheme.textMuted),
            filled: true,
            fillColor: UdayaTheme.bgSurface,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(UdayaTheme.inputRadius),
              borderSide: const BorderSide(color: UdayaTheme.borderColor, width: 1),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel', style: TextStyle(color: UdayaTheme.textTertiary)),
          ),
          FilledButton(
            onPressed: () {
              setState(() => _rpcEndpoint = controller.text.trim());
              _saveSettings();
              Navigator.pop(context);
            },
            style: FilledButton.styleFrom(
              backgroundColor: UdayaTheme.primaryColor,
              foregroundColor: UdayaTheme.textPrimary,
            ),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _showAddressTypeDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: UdayaTheme.bgCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(UdayaTheme.dialogRadius),
          side: const BorderSide(color: UdayaTheme.borderColorStrong, width: 1),
        ),
        title: const Text('Address Type', style: TextStyle(color: UdayaTheme.textPrimary, fontWeight: FontWeight.w600)),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: ['native', 'segwit', 'legacy'].map((type) {
            return RadioListTile<String>(
              title: Text(_addressTypeLabel(type), style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
              value: type,
              groupValue: _addressType,
              onChanged: (value) {
                if (value != null) {
                  setState(() => _addressType = value);
                  _saveSettings();
                  Navigator.pop(context);
                }
              },
              activeColor: UdayaTheme.primaryColor,
              contentPadding: const EdgeInsets.symmetric(horizontal: 0, vertical: 2),
              dense: true,
            );
          }).toList(),
        ),
      ),
    );
  }

  void _showCurrencyDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: UdayaTheme.bgCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(UdayaTheme.dialogRadius),
          side: const BorderSide(color: UdayaTheme.borderColorStrong, width: 1),
        ),
        title: const Text('Currency', style: TextStyle(color: UdayaTheme.textPrimary, fontWeight: FontWeight.w600)),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: ['USD', 'EUR', 'GBP', 'JPY'].map((currency) {
            return RadioListTile<String>(
              title: Text(currency, style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
              value: currency,
              groupValue: _selectedCurrency,
              onChanged: (value) {
                if (value != null) {
                  setState(() => _selectedCurrency = value);
                  _saveSettings();
                  Navigator.pop(context);
                }
              },
              activeColor: UdayaTheme.primaryColor,
              contentPadding: const EdgeInsets.symmetric(horizontal: 0, vertical: 2),
              dense: true,
            );
          }).toList(),
        ),
      ),
    );
  }

  void _showLogLevelDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: UdayaTheme.bgCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(UdayaTheme.dialogRadius),
          side: const BorderSide(color: UdayaTheme.borderColorStrong, width: 1),
        ),
        title: const Text('Log Level', style: TextStyle(color: UdayaTheme.textPrimary, fontWeight: FontWeight.w600)),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: ['verbose', 'debug', 'info', 'warning', 'error'].map((level) {
            return RadioListTile<String>(
              title: Text(level.toUpperCase(), style: TextStyle(color: UdayaTheme.textPrimary, fontSize: 14)),
              value: level,
              groupValue: _logLevel,
              onChanged: (value) {
                if (value != null) {
                  setState(() => _logLevel = value);
                  _saveSettings();
                  Navigator.pop(context);
                }
              },
              activeColor: UdayaTheme.primaryColor,
              contentPadding: const EdgeInsets.symmetric(horizontal: 0, vertical: 2),
              dense: true,
            );
          }).toList(),
        ),
      ),
    );
  }

  void _confirmClearCache() {
    GlassDialog.show(
      context: context,
      title: 'Clear Cache',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'This will clear all cached data including transaction history and blockchain data. Your wallet and funds will not be affected.',
            style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14, height: 1.5),
          ),
          const SizedBox(height: 20),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: Text('Cancel', style: TextStyle(color: UdayaTheme.textTertiary)),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: () async {
                  Navigator.pop(context);
                  final prefs = ref.read(sharedPrefsSourceProvider);
                  await prefs.clear();
                  if (!mounted) return;
                  setState(() {});
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Cache cleared successfully'),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                },
                style: FilledButton.styleFrom(
                  backgroundColor: UdayaTheme.warning,
                  foregroundColor: UdayaTheme.textPrimary,
                ),
                child: const Text('Clear Cache'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _confirmResetSettings() {
    GlassDialog.show(
      context: context,
      title: 'Reset Settings',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'This will reset all settings to their default values. This action cannot be undone.',
            style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14, height: 1.5),
          ),
          const SizedBox(height: 20),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: Text('Cancel', style: TextStyle(color: UdayaTheme.textTertiary)),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: () async {
                  Navigator.pop(context);
                  final prefs = ref.read(sharedPrefsSourceProvider);
                  await prefs.clear();
                  if (!mounted) return;
                  await _loadSettings();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Settings reset to defaults'),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                },
                style: FilledButton.styleFrom(
                  backgroundColor: UdayaTheme.error,
                  foregroundColor: UdayaTheme.textPrimary,
                ),
                child: const Text('Reset'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _SettingsSection extends StatelessWidget {
  final String title;
  final String subtitle;
  final IconData icon;
  final Color iconColor;
  final List<Widget> children;

  const _SettingsSection({
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.iconColor,
    required this.children,
  });

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      padding: EdgeInsets.zero,
      child: ExpansionTile(
        tilePadding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
        childrenPadding: const EdgeInsets.fromLTRB(0, 0, 0, 12),
        iconColor: UdayaTheme.textTertiary,
        collapsedIconColor: UdayaTheme.textTertiary,
        expandedCrossAxisAlignment: CrossAxisAlignment.end,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(UdayaTheme.cardRadius)),
        collapsedShape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(UdayaTheme.cardRadius)),
        leading: Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: iconColor.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(icon, color: iconColor, size: 18),
        ),
        title: Text(
          title,
          style: TextStyle(
            color: UdayaTheme.textPrimary,
            fontSize: 16,
            fontWeight: FontWeight.w600,
            letterSpacing: -0.2,
          ),
        ),
        subtitle: Text(
          subtitle,
          style: TextStyle(
            color: UdayaTheme.textTertiary,
            fontSize: 12,
          ),
        ),
        children: children,
      ),
    );
  }
}

class _SettingsTile extends StatelessWidget {
  final String title;
  final String subtitle;
  final IconData icon;
  final Color? iconColor;
  final Color? titleColor;
  final Widget? trailing;
  final VoidCallback? onTap;

  const _SettingsTile({
    required this.title,
    required this.subtitle,
    required this.icon,
    this.iconColor,
    this.titleColor,
    this.trailing,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveIconColor = iconColor ?? UdayaTheme.textTertiary;
    final effectiveTitleColor = titleColor ?? UdayaTheme.textPrimary;

    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(UdayaTheme.cardRadius),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
        child: Row(
          children: [
            Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: effectiveIconColor.withValues(alpha: 0.12),
                borderRadius: BorderRadius.circular(10),
              ),
              child: Icon(icon, color: effectiveIconColor, size: 18),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: TextStyle(
                      color: effectiveTitleColor,
                      fontSize: 15,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    subtitle,
                    style: TextStyle(
                      color: UdayaTheme.textTertiary,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
            if (trailing != null) ...[
              const SizedBox(width: 12),
              trailing!,
            ],
          ],
        ),
      ),
    );
  }
}

class _AboutRow extends StatelessWidget {
  final String label;
  final String value;

  const _AboutRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: UdayaTheme.textSecondary, fontSize: 14)),
          Text(
            value,
            style: const TextStyle(
              color: UdayaTheme.textPrimary,
              fontWeight: FontWeight.w500,
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }
}

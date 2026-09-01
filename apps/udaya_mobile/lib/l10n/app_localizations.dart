import 'package:flutter/material.dart';

class AppLocalizations {
  AppLocalizations(this.locale);

  final Locale locale;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const Map<String, Map<String, String>> _localizedValues = {
    'en': {
      'appTitle': 'Udaya',
      'createWallet': 'Create New Wallet',
      'restoreWallet': 'Restore Existing Wallet',
      'send': 'Send',
      'receive': 'Receive',
      'transactions': 'Transactions',
      'settings': 'Settings',
      'wallet': 'Wallet',
      'node': 'Node',
      'mining': 'Mining',
      'network': 'Network',
      'balance': 'Balance',
      'address': 'Address',
      'amount': 'Amount',
      'fee': 'Fee',
      'label': 'Label',
      'password': 'Password',
      'confirm': 'Confirm',
      'cancel': 'Cancel',
      'save': 'Save',
      'delete': 'Delete',
      'copy': 'Copy',
      'copied': 'Copied',
      'error': 'Error',
      'success': 'Success',
      'loading': 'Loading...',
      'noWalletFound': 'No Wallet Found',
      'createWalletDescription': 'Create a new wallet to get started.',
      'recipientAddress': 'Recipient Address',
      'enterAddress': 'Enter Udaya address',
      'scanQRCode': 'Scan QR Code',
      'generatePaymentRequest': 'Generate Payment Request',
      'availableBalance': 'Available Balance',
      'sendTransaction': 'Send Transaction',
      'transactionSent': 'Transaction sent successfully',
      'transactionFailed': 'Transaction failed',
      'insufficientBalance': 'Insufficient balance',
      'invalidAddress': 'Invalid address',
      'invalidAmount': 'Invalid amount',
      'seedPhrase': 'Seed Phrase',
      'backupSeedPhrase': 'Backup Seed Phrase',
      'neverShareSeed': 'Never share your seed phrase with anyone',
      'security': 'Security',
      'biometricAuth': 'Biometric Authentication',
      'passphrase': 'Passphrase',
      'lockWallet': 'Lock Wallet',
      'unlockWallet': 'Unlock Wallet',
      'changePassphrase': 'Change Passphrase',
      'language': 'Language',
      'theme': 'Theme',
      'darkMode': 'Dark Mode',
      'lightMode': 'Light Mode',
      'systemMode': 'System',
      'about': 'About',
      'version': 'Version',
      'privacyPolicy': 'Privacy Policy',
      'termsOfService': 'Terms of Service',
      'ok': 'OK',
    },
  };

  String get(String key) {
    return _localizedValues[locale.languageCode]?[key] ??
        _localizedValues['en']?[key] ??
        key;
  }

  String get appTitle => get('appTitle');
  String get createWallet => get('createWallet');
  String get restoreWallet => get('restoreWallet');
  String get send => get('send');
  String get receive => get('receive');
  String get transactions => get('transactions');
  String get settings => get('settings');
  String get wallet => get('wallet');
  String get node => get('node');
  String get mining => get('mining');
  String get network => get('network');
  String get balance => get('balance');
  String get address => get('address');
  String get amount => get('amount');
  String get fee => get('fee');
  String get label => get('label');
  String get password => get('password');
  String get confirm => get('confirm');
  String get cancel => get('cancel');
  String get save => get('save');
  String get delete => get('delete');
  String get copy => get('copy');
  String get copied => get('copied');
  String get error => get('error');
  String get success => get('success');
  String get loading => get('loading');
  String get noWalletFound => get('noWalletFound');
  String get createWalletDescription => get('createWalletDescription');
  String get recipientAddress => get('recipientAddress');
  String get enterAddress => get('enterAddress');
  String get scanQRCode => get('scanQRCode');
  String get generatePaymentRequest => get('generatePaymentRequest');
  String get availableBalance => get('availableBalance');
  String get sendTransaction => get('sendTransaction');
  String get transactionSent => get('transactionSent');
  String get transactionFailed => get('transactionFailed');
  String get insufficientBalance => get('insufficientBalance');
  String get invalidAddress => get('invalidAddress');
  String get invalidAmount => get('invalidAmount');
  String get seedPhrase => get('seedPhrase');
  String get backupSeedPhrase => get('backupSeedPhrase');
  String get neverShareSeed => get('neverShareSeed');
  String get security => get('security');
  String get biometricAuth => get('biometricAuth');
  String get passphrase => get('passphrase');
  String get lockWallet => get('lockWallet');
  String get unlockWallet => get('unlockWallet');
  String get changePassphrase => get('changePassphrase');
  String get language => get('language');
  String get theme => get('theme');
  String get darkMode => get('darkMode');
  String get lightMode => get('lightMode');
  String get systemMode => get('systemMode');
  String get about => get('about');
  String get version => get('version');
  String get privacyPolicy => get('privacyPolicy');
  String get termsOfService => get('termsOfService');
  String get ok => get('ok');
}

class AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const AppLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) => ['en'].contains(locale.languageCode);

  @override
  Future<AppLocalizations> load(Locale locale) async {
    return AppLocalizations(locale);
  }

  @override
  bool shouldReload(AppLocalizationsDelegate old) => false;
}

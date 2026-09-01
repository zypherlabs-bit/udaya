class AppConstants {
  static const String appName = 'Udaya';
  static const String appTagline = 'Native Layer-1 Blockchain';
  static const String version = '1.0.0-alpha.1';

  static const String mainnet = 'mainnet';
  static const String testnet = 'testnet';
  static const String regtest = 'regtest';

  static const int defaultPort = 9798;
  static const int defaultRpcPort = 8332;
  static const int defaultTestnetPort = 19798;
  static const int defaultTestnetRpcPort = 18332;

  static const String defaultDataDir = 'data/udaya';
  static const String defaultTestnetDataDir = 'temp_testnet_data';

  static const int satoshisPerCoin = 100000000;
  static const int maxBlockSize = 1000000;
  static const int maxBlockWeight = 4000000;
  static const int coinbaseMaturity = 100;
  static const int blockTargetTimeSecs = 600;

  static const int defaultFeeRate = 10;
  static const int minFeeRate = 1;
  static const int maxFeeRate = 1000;

  static const String mainnetAddressPrefix = 'U';
  static const String testnetAddressPrefix = 'U';

  static const String networkMagicHex = 'BF591AE7';
}

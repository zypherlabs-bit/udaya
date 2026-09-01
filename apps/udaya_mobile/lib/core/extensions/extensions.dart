import 'package:udaya_mobile/core/constants/app_constants.dart';

extension DoubleExtension on double {
  double get toUdya => this / AppConstants.satoshisPerCoin;
}

extension IntExtension on int {
  double get toUdya => this / AppConstants.satoshisPerCoin;

  String get toUdyaString {
    final whole = this ~/ AppConstants.satoshisPerCoin;
    final fraction = this % AppConstants.satoshisPerCoin;
    return '$whole.$fraction';
  }

  String get toUdyaFormatted {
    final value = this.toUdya;
    return value.toStringAsFixed(8);
  }
}

extension StringExtension on String {
  String get truncateMiddle {
    if (length <= 16) return this;
    return '${substring(0, 8)}...${substring(length - 8)}';
  }

  bool get isValidHex {
    final regex = RegExp(r'^[0-9a-fA-F]+$');
    return regex.hasMatch(this) && length % 2 == 0;
  }
}

extension DateTimeExtension on DateTime {
  String get toRelativeTime {
    final now = DateTime.now();
    final diff = now.difference(this);

    if (diff.inSeconds < 60) return 'Just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    if (diff.inDays < 7) return '${diff.inDays}d ago';
    return toLocal().toString().split(' ')[0];
  }
}

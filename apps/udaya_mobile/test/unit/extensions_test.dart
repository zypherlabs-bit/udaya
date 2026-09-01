import 'package:flutter_test/flutter_test.dart';
import 'package:udaya_mobile/core/extensions/extensions.dart';

void main() {
  group('IntExtension', () {
    test('should convert satoshis to UDYA', () {
      expect(100000000.toUdya, 1.0);
      expect(50000000.toUdya, 0.5);
      expect(0.toUdya, 0.0);
    });

    test('should format satoshis to UDYA string', () {
      expect(100000000.toUdyaString, '1.0');
      expect(150000000.toUdyaString, '1.50000000');
    });

    test('should format satoshis to formatted UDYA', () {
      expect(100000000.toUdyaFormatted, '1.00000000');
      expect(12345678.toUdyaFormatted, '0.12345678');
    });
  });

  group('StringExtension', () {
    test('should truncate middle of long strings', () {
      expect('12345678901234567890'.truncateMiddle, '12345678...34567890');
    });

    test('should not truncate short strings', () {
      expect('short'.truncateMiddle, 'short');
    });

    test('should validate hex strings', () {
      expect('deadbeef'.isValidHex, true);
      expect('DEADBEEF'.isValidHex, true);
      expect('xyz'.isValidHex, false);
      expect('abc'.isValidHex, false);
    });
  });
}

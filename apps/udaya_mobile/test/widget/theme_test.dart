import 'package:flutter_test/flutter_test.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

void main() {
  testWidgets('UdayaTheme should have correct colors', (WidgetTester tester) async {
    final theme = UdayaTheme.darkTheme;
    expect(theme.colorScheme.primary, UdayaTheme.primaryColor);
    expect(theme.scaffoldBackgroundColor, UdayaTheme.bgDeep);
  });

  testWidgets('Material3 should be enabled', (WidgetTester tester) async {
    final theme = UdayaTheme.darkTheme;
    expect(theme.useMaterial3, true);
  });
}

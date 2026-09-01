import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class UdayaTheme {
  static const String fontFamily = 'Inter';

  static const Color primaryColor = Color(0xFF06B6D4);
  static const Color primaryGlow = Color(0x4006B6D4);
  static const Color primaryLight = Color(0xFF22D3EE);
  static const Color primaryDark = Color(0xFF0891B2);
  static const Color secondaryColor = Color(0xFF0EA5E9);
  static const Color secondaryGlow = Color(0x400EA5E9);
  static const Color accentColor = Color(0xFF22D3EE);
  static const Color accentGlow = Color(0x4022D3EE);

  static const Color bgDeep = Color(0xFF050510);
  static const Color bgDark = Color(0xFF0A0A1A);
  static const Color bgSurface = Color(0xFF0F0F23);
  static const Color bgCard = Color(0xFF141430);
  static const Color bgCardHover = Color(0xFF1A1A3E);
  static const Color bgGlass = Color(0x14142830);
  static const Color bgGlassLight = Color(0x1AFFFFFF);
  static const Color bgOverlay = Color(0x99050510);

  static const Color textPrimary = Color(0xFFF8FAFC);
  static const Color textSecondary = Color(0xFFCBD5E1);
  static const Color textTertiary = Color(0xFF64748B);
  static const Color textMuted = Color(0xFF475569);
  static const Color textInverse = Color(0xFF0F172A);

  static const Color borderColor = Color(0x1AFFFFFF);
  static const Color borderColorStrong = Color(0x33FFFFFF);
  static const Color dividerColor = Color(0x0DFFFFFF);

  static const Color success = Color(0xFF06B6D4);
  static const Color successGlow = Color(0x4006B6D4);
  static const Color warning = Color(0xFFF59E0B);
  static const Color warningGlow = Color(0x40F59E0B);
  static const Color error = Color(0xFFEF4444);
  static const Color errorGlow = Color(0x40EF4444);
  static const Color info = Color(0xFF0EA5E9);
  static const Color infoGlow = Color(0x400EA5E9);

  static const Color mainnetColor = Color(0xFF06B6D4);
  static const Color testnetColor = Color(0xFFF59E0B);
  static const Color regtestColor = Color(0xFF0EA5E9);
  static const Color syncingColor = Color(0xFFF59E0B);
  static const Color unconfirmedColor = Color(0xFFEF4444);
  static const Color confirmedColor = Color(0xFF06B6D4);

  static const double cardRadius = 20;
  static const double panelRadius = 24;
  static const double buttonRadius = 14;
  static const double chipRadius = 10;
  static const double inputRadius = 12;
  static const double fabRadius = 16;
  static const double dialogRadius = 28;
  static const double bottomSheetRadius = 28;

  static const double spacingXs = 4;
  static const double spacingSm = 8;
  static const double spacingMd = 16;
  static const double spacingLg = 24;
  static const double spacingXl = 32;
  static const double spacing2xl = 48;

  static const Duration animationFast = Duration(milliseconds: 150);
  static const Duration animationNormal = Duration(milliseconds: 250);
  static const Duration animationSlow = Duration(milliseconds: 350);
  static const Duration animationSpring = Duration(milliseconds: 400);
  static const Curve animationCurve = Curves.easeInOutCubic;
  static const Curve animationCurveEmphasized = Curves.easeInOutBack;

  static TextTheme get textTheme {
    return TextTheme(
      displayLarge: GoogleFonts.inter(fontSize: 48, fontWeight: FontWeight.w200, color: textPrimary, height: 1.1, letterSpacing: -1.5),
      displayMedium: GoogleFonts.inter(fontSize: 36, fontWeight: FontWeight.w300, color: textPrimary, height: 1.15, letterSpacing: -1),
      displaySmall: GoogleFonts.inter(fontSize: 28, fontWeight: FontWeight.w400, color: textPrimary, height: 1.2, letterSpacing: -0.5),
      headlineLarge: GoogleFonts.inter(fontSize: 24, fontWeight: FontWeight.w500, color: textPrimary, height: 1.3),
      headlineMedium: GoogleFonts.inter(fontSize: 20, fontWeight: FontWeight.w500, color: textPrimary, height: 1.35),
      headlineSmall: GoogleFonts.inter(fontSize: 18, fontWeight: FontWeight.w600, color: textPrimary, height: 1.4),
      titleLarge: GoogleFonts.inter(fontSize: 16, fontWeight: FontWeight.w600, color: textPrimary, height: 1.45),
      titleMedium: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w600, color: textPrimary, height: 1.5),
      titleSmall: GoogleFonts.inter(fontSize: 13, fontWeight: FontWeight.w600, color: textSecondary, height: 1.5),
      bodyLarge: GoogleFonts.inter(fontSize: 15, fontWeight: FontWeight.w400, color: textSecondary, height: 1.6),
      bodyMedium: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w400, color: textSecondary, height: 1.6),
      bodySmall: GoogleFonts.inter(fontSize: 13, fontWeight: FontWeight.w400, color: textTertiary, height: 1.6),
      labelLarge: GoogleFonts.inter(fontSize: 13, fontWeight: FontWeight.w500, color: textSecondary, height: 1.5, letterSpacing: 0.5),
      labelMedium: GoogleFonts.inter(fontSize: 12, fontWeight: FontWeight.w500, color: textTertiary, height: 1.5, letterSpacing: 0.3),
      labelSmall: GoogleFonts.inter(fontSize: 11, fontWeight: FontWeight.w500, color: textMuted, height: 1.5, letterSpacing: 0.5),
    );
  }

  static ThemeData get darkTheme {
    final base = ThemeData.dark(useMaterial3: true);
    return base.copyWith(
      textTheme: textTheme,
      colorScheme: const ColorScheme.dark(
        primary: primaryColor,
        primaryContainer: bgCard,
        secondary: secondaryColor,
        tertiary: accentColor,
        surface: bgSurface,
        surfaceContainerHighest: bgCard,
        background: bgDeep,
        error: error,
        onPrimary: textPrimary,
        onSecondary: textPrimary,
        onSurface: textPrimary,
        onBackground: textPrimary,
        onError: textPrimary,
        outline: borderColor,
        outlineVariant: borderColorStrong,
      ),
      scaffoldBackgroundColor: bgDeep,
      appBarTheme: AppBarTheme(
        backgroundColor: Colors.transparent,
        foregroundColor: textPrimary,
        elevation: 0,
        centerTitle: false,
        titleTextStyle: GoogleFonts.inter(fontSize: 20, fontWeight: FontWeight.w600, color: textPrimary),
      ),
      cardTheme: CardThemeData(
        color: bgCard,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(cardRadius),
          side: const BorderSide(color: borderColor, width: 1),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primaryColor,
          foregroundColor: textPrimary,
          elevation: 0,
          shadowColor: primaryGlow,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(buttonRadius),
            side: const BorderSide(color: borderColorStrong, width: 1),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: secondaryColor,
          side: const BorderSide(color: secondaryColor, width: 1.5),
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(buttonRadius),
          ),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: bgSurface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: borderColor, width: 1),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: primaryColor, width: 1.5),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: error, width: 1),
        ),
        focusedErrorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: error, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        labelStyle: const TextStyle(color: textTertiary),
        hintStyle: const TextStyle(color: textMuted),
      ),
      dividerTheme: const DividerThemeData(
        color: dividerColor,
        thickness: 1,
        space: 1,
      ),
      switchTheme: SwitchThemeData(
        thumbColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) return primaryColor;
          return textMuted;
        }),
        trackColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) return primaryColor.withValues(alpha: 0.3);
          return textMuted.withValues(alpha: 0.2);
        }),
      ),
      sliderTheme: SliderThemeData(
        activeTrackColor: primaryColor,
        inactiveTrackColor: textMuted.withValues(alpha: 0.2),
        thumbColor: primaryColor,
        overlayColor: primaryGlow,
        trackHeight: 4,
        thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 10),
      ),
      bottomSheetTheme: const BottomSheetThemeData(
        backgroundColor: bgCard,
        surfaceTintColor: bgCard,
        elevation: 8,
      ),
      dialogTheme: DialogThemeData(
        backgroundColor: bgCard,
        elevation: 8,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(dialogRadius),
          side: const BorderSide(color: borderColor, width: 1),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: bgCard,
        contentTextStyle: const TextStyle(color: textPrimary, fontSize: 14),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(chipRadius),
          side: const BorderSide(color: borderColor, width: 1),
        ),
        behavior: SnackBarBehavior.floating,
      ),
      floatingActionButtonTheme: FloatingActionButtonThemeData(
        backgroundColor: primaryColor,
        foregroundColor: textPrimary,
        elevation: 4,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(fabRadius),
        ),
      ),
      tooltipTheme: TooltipThemeData(
        decoration: BoxDecoration(
          color: bgCard,
          borderRadius: BorderRadius.circular(chipRadius),
          border: Border.all(color: borderColorStrong, width: 1),
        ),
        textStyle: const TextStyle(color: textPrimary, fontSize: 12),
      ),
    );
  }

  static ThemeData get lightTheme {
    final base = ThemeData.light(useMaterial3: true);
    return base.copyWith(
      textTheme: textTheme.apply(bodyColor: const Color(0xFF1E293B), displayColor: const Color(0xFF0F172A)),
      colorScheme: ColorScheme.light(
        primary: primaryColor,
        primaryContainer: const Color(0xFFEEF2FF),
        secondary: secondaryColor,
        tertiary: accentColor,
        surface: Colors.white,
        surfaceContainerHighest: const Color(0xFFF1F5F9),
        background: const Color(0xFFF8FAFC),
        error: error,
        onPrimary: Colors.white,
        onSecondary: Colors.white,
        onSurface: const Color(0xFF1E293B),
        onBackground: const Color(0xFF1E293B),
        onError: Colors.white,
        outline: const Color(0xFFE2E8F0),
        outlineVariant: const Color(0xFFCBD5E1),
      ),
      scaffoldBackgroundColor: const Color(0xFFF8FAFC),
      appBarTheme: AppBarTheme(
        backgroundColor: Colors.white,
        foregroundColor: primaryColor,
        elevation: 0,
        centerTitle: false,
        titleTextStyle: GoogleFonts.inter(fontSize: 20, fontWeight: FontWeight.w600, color: const Color(0xFF1E293B)),
      ),
      cardTheme: CardThemeData(
        color: Colors.white,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(cardRadius),
          side: const BorderSide(color: Color(0xFFE2E8F0), width: 1),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primaryColor,
          foregroundColor: Colors.white,
          elevation: 0,
          shadowColor: primaryGlow,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(buttonRadius),
            side: const BorderSide(color: Color(0xFFE2E8F0), width: 1),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: primaryColor,
          side: const BorderSide(color: primaryColor, width: 1.5),
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(buttonRadius),
          ),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: const Color(0xFFF1F5F9),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: Color(0xFFE2E8F0), width: 1),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(inputRadius),
          borderSide: const BorderSide(color: primaryColor, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        labelStyle: const TextStyle(color: Color(0xFF64748B)),
        hintStyle: const TextStyle(color: Color(0xFF94A3B8)),
      ),
      dividerTheme: const DividerThemeData(
        color: Color(0xFFE2E8F0),
        thickness: 1,
        space: 1,
      ),
      bottomSheetTheme: const BottomSheetThemeData(
        backgroundColor: Colors.white,
        surfaceTintColor: Colors.white,
        elevation: 8,
      ),
      dialogTheme: DialogThemeData(
        backgroundColor: Colors.white,
        elevation: 8,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(dialogRadius),
          side: const BorderSide(color: Color(0xFFE2E8F0), width: 1),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: Colors.white,
        contentTextStyle: const TextStyle(color: Color(0xFF1E293B), fontSize: 14),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(chipRadius),
          side: const BorderSide(color: Color(0xFFE2E8F0), width: 1),
        ),
        behavior: SnackBarBehavior.floating,
      ),
      floatingActionButtonTheme: FloatingActionButtonThemeData(
        backgroundColor: primaryColor,
        foregroundColor: Colors.white,
        elevation: 4,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(fabRadius),
        ),
      ),
    );
  }

  static BoxDecoration glassDecoration({
    Color? color,
    double blur = 12,
    double opacity = 0.15,
    BorderRadius? borderRadius,
    Border? border,
    Gradient? gradient,
    List<BoxShadow>? shadows,
  }) {
    final effectiveBorder = border ??
        Border.all(
          color: UdayaTheme.borderColorStrong,
          width: 1,
        );
    final effectiveBorderRadius = borderRadius ??
        BorderRadius.circular(cardRadius);
    final effectiveGradient = gradient ??
        LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            (color ?? UdayaTheme.primaryColor).withValues(alpha: opacity),
            (color ?? UdayaTheme.accentColor).withValues(alpha: opacity * 0.5),
          ],
        );
    final effectiveShadows = shadows ?? [
      BoxShadow(
        color: (color ?? UdayaTheme.primaryColor).withValues(alpha: 0.1),
        blurRadius: 20,
        offset: const Offset(0, 4),
      ),
    ];

    return BoxDecoration(
      gradient: effectiveGradient,
      borderRadius: effectiveBorderRadius,
      border: effectiveBorder,
      boxShadow: effectiveShadows,
    );
  }

  static Widget glassOverlay({
    required Widget child,
    double blur = 12,
    Color? color,
    double opacity = 0.1,
  }) {
    return BackdropFilter(
      filter: ImageFilter.blur(sigmaX: blur, sigmaY: blur),
      child: child,
    );
  }

  static TextStyle get balanceTextStyle {
    return const TextStyle(
      fontSize: 28,
      fontWeight: FontWeight.w700,
      color: textPrimary,
      letterSpacing: -0.8,
      height: 1.2,
    );
  }

  static TextStyle get balanceSmallTextStyle {
    return const TextStyle(
      fontSize: 18,
      fontWeight: FontWeight.w600,
      color: textPrimary,
      letterSpacing: -0.5,
      height: 1.3,
    );
  }

  static TextStyle get labelTextStyle {
    return const TextStyle(
      fontSize: 12,
      fontWeight: FontWeight.w500,
      color: textTertiary,
      letterSpacing: 0.5,
      height: 1.5,
    );
  }

  static TextStyle get chipTextStyle {
    return const TextStyle(
      fontSize: 12,
      fontWeight: FontWeight.w500,
      color: textSecondary,
      letterSpacing: 0.3,
    );
  }

  static TextStyle get errorTextStyle {
    return const TextStyle(
      fontSize: 13,
      fontWeight: FontWeight.w400,
      color: error,
      height: 1.5,
    );
  }

  static TextStyle get successTextStyle {
    return const TextStyle(
      fontSize: 13,
      fontWeight: FontWeight.w500,
      color: success,
      height: 1.5,
    );
  }
}

extension ColorExtension on Color {
  Color withValues({required double alpha}) {
    return Color.fromRGBO(
      (red * 255).round(),
      (green * 255).round(),
      (blue * 255).round(),
      alpha,
    );
  }

  Color withOpacity(double opacity) {
    return withValues(alpha: opacity);
  }

  Color darken([double amount = 0.1]) {
    final hsl = HSLColor.fromColor(this);
    return hsl.withLightness((hsl.lightness - amount).clamp(0.0, 1.0)).toColor();
  }

  Color lighten([double amount = 0.1]) {
    final hsl = HSLColor.fromColor(this);
    return hsl.withLightness((hsl.lightness + amount).clamp(0.0, 1.0)).toColor();
  }
}

extension TextStyleExtension on TextStyle {
  TextStyle withColor(Color color) {
    return copyWith(color: color);
  }

  TextStyle withWeight(FontWeight weight) {
    return copyWith(fontWeight: weight);
  }

  TextStyle withSize(double size) {
    return copyWith(fontSize: size);
  }
}

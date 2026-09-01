# Udaya Mobile - Defect Report

Discovered during the UI/UX transformation to futuristic glassmorphism design.

## Critical Issues (None)

## High Priority

### Issue: Biometric preference stored in insecure SharedPreferences
**File**: `lib/services/biometric_service.dart`
**Description**: The biometric authentication enabled flag was stored in `SharedPreferences` instead of secure storage.
**Risk**: An attacker with root access could modify the SharedPreferences file to bypass biometric authentication.
**Fix**: Migrated biometric preferences to `FlutterSecureStorage` with `AndroidOptions(encryptedSharedPreferences: true)`.
**Labels**: `security`, `android`, `critical`

## Medium Priority

### Issue: Missing Android core library desugaring
**File**: `android/app/build.gradle.kts`
**Description**: The `flutter_local_notifications` dependency requires Java 8+ desugaring but the build configuration did not enable it.
**Risk**: Release build fails with `checkReleaseAarMetadata` error.
**Fix**: Added `isCoreLibraryDesugaringEnabled = true` in `compileOptions` and added `coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.1.2")` dependency.
**Labels**: `android`, `build`

### Issue: App bar pattern inconsistent across screens
**File**: Multiple screen files
**Description**: Screens were using a mix of `Scaffold` with `appBar`, `SliverAppBar`, and custom app bars without consistent glassmorphism.
**Risk**: Inconsistent user experience across the app.
**Fix**: Standardized on `GlassAppBar` with `CustomScrollView` + `SliverAppBar` pattern for all screens with scrollable content.
**Labels**: `ui`, `android`

## Low Priority

### Issue: Widget tests referencing old BalanceCard widget
**File**: `test/widget/screens_test.dart`
**Description**: Tests expected `BalanceCard` widget but the new design uses inline balance sections with `GlassCard`.
**Risk**: Tests fail after UI transformation.
**Fix**: Updated tests to look for `GlassCard` instead of `BalanceCard`.
**Labels**: `test`, `android`

### Issue: Rust dependency warnings in cargo audit
**File**: `Cargo.toml` dependencies
**Description**: `rustls-pemfile` and `atty` dependencies have security advisories (unmaintained, potential unaligned read).
**Risk**: These are transitive dependencies, not directly used in the Udaya core blockchain logic.
**Fix**: Update dependencies when new versions are released. These are pre-existing issues.
**Labels**: `security`, `performance`

## Resolved During Transformation

All issues below were discovered and resolved during the transformation:

1. `Card` widget replaced with `GlassCard` across all screens
2. `AlertDialog` replaced with `GlassDialog` across all screens
3. `showModalBottomSheet` replaced with `GlassBottomSheet.show()` 
4. `ElevatedButton` replaced with `GlassButton` for primary actions
5. `OutlinedButton` replaced with `GlassButton(isOutlined: true)` for secondary actions
6. `NavigationBar` replaced with `GlassNavigationBar` in app shell
7. Status indicators upgraded from simple text chips to `StatusIndicator` with pulse animation
8. Balance displays upgraded to animated number transitions
9. All Scaffold background colors updated to `UdayaTheme.bgDeep`
10. Transaction status displays upgraded to `TransactionCard` with visual states

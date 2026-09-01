# Flutter/Dart code
-keep class io.flutter.app.** { *; }
-keep class io.flutter.embedding.** { *; }
-keep class io.flutter.plugins.** { *; }
-keep class io.flutter.util.** { *; }
-keep class io.flutter.view.** { *; }
-keep class io.flutter.plugin.common.** { *; }
-keep class io.flutter.plugin.registrar.** { *; }
-keep class io.flutter.plugin.platform.** { *; }

# GeneratedPluginRegistrant
-keep class io.flutter.plugins.GeneratedPluginRegistrant { *; }

# Riverpod
-keep class flutter_riverpod.** { *; }
-keep class riverpod.** { *; }

# GoRouter
-keep class go_router.** { *; }

# QR code scanning
-keep class com.it_nomads.fluttersecurestorage.** { *; }
-keep class dev.steenbakker.mobile_scanner.** { *; }
-keep class net.touchcapture.qr.flutterqr.** { *; }

# JSON serialization
-keep class com.google.gson.** { *; }
-keepattributes *Annotation*

# JNI bridge
-keep class com.udaya.mobile.UdayaNativeBridge { *; }

# Keep native method signatures
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep custom exceptions
-keep class com.udaya.mobile.** { *; }

# Keep Flutter engine
-keep class io.flutter.engine.** { *; }
-keep class io.flutter.binding.** { *; }

# ProGuard/R8 general rules
-dontwarn io.flutter.embedding.**
-dontwarn io.flutter.plugins.**
-dontwarn io.flutter.util.**
-dontwarn io.flutter.view.**
-dontwarn io.flutter.plugin.common.**
-dontwarn io.flutter.plugin.registrar.**
-dontwarn io.flutter.plugin.platform.**
-dontwarn flutter_riverpod.**
-dontwarn riverpod.**
-dontwarn go_router.**
-dontwarn com.it_nomads.fluttersecurestorage.**
-dontwarn dev.steenbakker.mobile_scanner.**
-dontwarn net.touchcapture.qr.flutterqr.**
-dontwarn com.google.gson.**
-dontwarn com.udaya.mobile.**
-dontwarn io.flutter.engine.**
-dontwarn io.flutter.binding.**
-dontwarn javax.annotation.**

# Optimization
-optimizations !code/simplification/arithmetic,!field/*,!class/merging/*
-allowaccessmodification

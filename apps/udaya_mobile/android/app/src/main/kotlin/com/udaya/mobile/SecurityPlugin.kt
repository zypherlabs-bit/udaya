package com.udaya.mobile

import android.app.Activity
import android.view.WindowManager
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class SecurityPlugin(private val activity: Activity) : MethodChannel.MethodCallHandler {
    companion object {
        private const val CHANNEL = "udaya_mobile/security"

        fun registerWith(flutterEngine: FlutterEngine, activity: Activity) {
            val channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
            channel.setMethodCallHandler(SecurityPlugin(activity))
        }
    }

    override fun onMethodCall(call: io.flutter.plugin.common.MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "enableScreenshotProtection" -> {
                enableScreenshotProtection()
                result.success(null)
            }
            "disableScreenshotProtection" -> {
                disableScreenshotProtection()
                result.success(null)
            }
            else -> result.notImplemented()
        }
    }

    private fun enableScreenshotProtection() {
        activity.window.setFlags(
            WindowManager.LayoutParams.FLAG_SECURE,
            WindowManager.LayoutParams.FLAG_SECURE
        )
    }

    private fun disableScreenshotProtection() {
        activity.window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
    }
}

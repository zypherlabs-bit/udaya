package com.udaya.mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.dart.DartExecutor

class UdayaNodeService : Service() {
    private val binder = LocalBinder()
    private var flutterEngine: FlutterEngine? = null
    private var isRunning = false

    companion object {
        const val CHANNEL_ID = "udaya_node_service"
        const val NOTIFICATION_ID = 1001
        const val ACTION_START = "com.udaya.mobile.START_NODE"
        const val ACTION_STOP = "com.udaya.mobile.STOP_NODE"
    }

    inner class LocalBinder : Binder() {
        fun getService(): UdayaNodeService = this@UdayaNodeService
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startNode()
            ACTION_STOP -> stopNode()
        }
        return START_STICKY
    }

    override fun onBind(intent: Intent): IBinder {
        return binder
    }

    override fun onDestroy() {
        stopNode()
        flutterEngine?.destroy()
        super.onDestroy()
    }

    fun startNode() {
        if (isRunning) return

        val notification = buildNotification("Starting Udaya Node...")
        startForeground(NOTIFICATION_ID, notification)

        flutterEngine = FlutterEngine(this)
        flutterEngine?.dartExecutor?.executeDartEntrypoint(
            DartExecutor.DartEntrypoint.createDefault()
        )

        isRunning = true
    }

    fun stopNode() {
        if (!isRunning) return
        isRunning = false
        flutterEngine?.destroy()
        flutterEngine = null
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    fun isNodeRunning(): Boolean = isRunning

    private fun buildNotification(content: String): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this, 0,
            packageManager.getLaunchIntentForPackage(packageName),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Udaya Node")
            .setContentText(content)
            .setSmallIcon(android.R.drawable.stat_sys_data_bluetooth)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Udaya Node Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows Udaya node running status"
                setShowBadge(false)
            }

            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }
}

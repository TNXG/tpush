package moe.tnxg.push.core

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import org.json.JSONObject
import java.util.concurrent.atomic.AtomicInteger

object Notifications {
    fun buildForegroundNotification(context: Context, channelName: String): Notification {
        ensureNotificationChannel(context)
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setContentTitle("TPush")
            .setContentText("频道 $channelName 运行中")
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setLocalOnly(true)
            .setSilent(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .build()
    }

    fun showPushNotification(context: Context, messageJson: String) {
        ensureNotificationChannel(context)
        logNotificationState(context)
        val message = runCatching { JSONObject(messageJson) }.getOrNull()
        val title = message?.optString("title")?.takeIf { it.isNotBlank() } ?: "TPush"
        val content = message?.optString("content")?.takeIf { it.isNotBlank() } ?: "新消息"
        val messageId = message?.optString("id")?.takeIf { it.isNotBlank() } ?: content
        val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)
        val pendingIntent = PendingIntent.getActivity(
            context,
            messageId.hashCode(),
            launchIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or immutableFlag(),
        )

        val notification = NotificationCompat.Builder(context, MESSAGE_CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_notify_more)
            .setContentTitle(title)
            .setContentText(content)
            .setStyle(NotificationCompat.BigTextStyle().bigText(content))
            .setContentIntent(pendingIntent)
            .setAutoCancel(true)
            .setCategory(NotificationCompat.CATEGORY_MESSAGE)
            .setDefaults(NotificationCompat.DEFAULT_ALL)
            .setOnlyAlertOnce(false)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setPublicVersion(
                NotificationCompat.Builder(context, MESSAGE_CHANNEL_ID)
                    .setSmallIcon(android.R.drawable.stat_notify_more)
                    .setContentTitle(title)
                    .setContentText(content)
                    .build(),
            )
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setWhen(System.currentTimeMillis())
            .setShowWhen(true)
            .build()

        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        notificationManager.notify(nextMessageNotificationId(), notification)
    }

    private fun ensureNotificationChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            return
        }

        val notificationManager = context.getSystemService(NotificationManager::class.java)
        cleanupLegacyChannels(notificationManager)
        val keepAliveChannel = NotificationChannel(
            CHANNEL_ID,
            "TPush 保活服务",
            NotificationManager.IMPORTANCE_MIN,
        )
        keepAliveChannel.setShowBadge(false)
        notificationManager.createNotificationChannel(keepAliveChannel)

        val messageChannel = NotificationChannel(
            MESSAGE_CHANNEL_ID,
            "TPush 推送消息",
            NotificationManager.IMPORTANCE_HIGH,
        )
        notificationManager.createNotificationChannel(messageChannel)
    }

    private fun cleanupLegacyChannels(notificationManager: NotificationManager) {
        LEGACY_CHANNEL_IDS.forEach { channelId ->
            notificationManager.deleteNotificationChannel(channelId)
        }
    }

    private fun logNotificationState(context: Context) {
        val notificationsEnabled = NotificationManagerCompat.from(context).areNotificationsEnabled()
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            Log.i(LOG_TAG, "[NOTIFICATION_STATE] enabled=$notificationsEnabled")
            return
        }

        val notificationManager = context.getSystemService(NotificationManager::class.java)
        val channel = notificationManager.getNotificationChannel(MESSAGE_CHANNEL_ID)
        Log.i(
            LOG_TAG,
            "[NOTIFICATION_STATE] enabled=$notificationsEnabled channel=$MESSAGE_CHANNEL_ID importance=${channel?.importance ?: -1} sound=${channel?.sound ?: "null"} vibration=${channel?.shouldVibrate() ?: false}",
        )
    }

    private fun immutableFlag(): Int =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0

    private fun nextMessageNotificationId(): Int =
        MESSAGE_NOTIFICATION_BASE_ID + notificationSequence.getAndIncrement().mod(100_000)

    private const val CHANNEL_ID = "tpush.keepalive.service"
    private const val MESSAGE_CHANNEL_ID = "tpush.messages.default"
    private const val MESSAGE_NOTIFICATION_BASE_ID = 3000
    private const val LOG_TAG = "TPush"
    private val LEGACY_CHANNEL_IDS = listOf(
        "tpush.keepalive",
        "tpush.keepalive.v2",
        "tpush.keepalive.default",
        "tpush.messages",
        "tpush.messages.alerts",
        "tpush.messages.sound.v2",
    )
    private val notificationSequence = AtomicInteger((System.currentTimeMillis() % 100_000).toInt())
}

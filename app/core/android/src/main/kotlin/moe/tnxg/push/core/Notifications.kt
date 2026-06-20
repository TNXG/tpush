package moe.tnxg.push.core

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat
import org.json.JSONObject

object Notifications {
    fun buildForegroundNotification(context: Context, channelName: String): Notification {
        ensureNotificationChannel(context)
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setContentTitle("TPush")
            .setContentText("频道 $channelName 运行中")
            .setOngoing(true)
            .setSilent(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .build()
    }

    fun showPushNotification(context: Context, messageJson: String) {
        ensureNotificationChannel(context)
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
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()

        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        notificationManager.notify(
            MESSAGE_NOTIFICATION_BASE_ID + messageId.hashCode().mod(100_000),
            notification,
        )
    }

    private fun ensureNotificationChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            return
        }

        val notificationManager = context.getSystemService(NotificationManager::class.java)
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
            NotificationManager.IMPORTANCE_DEFAULT,
        )
        messageChannel.setShowBadge(true)
        notificationManager.createNotificationChannel(messageChannel)
    }

    private fun immutableFlag(): Int =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0

    private const val CHANNEL_ID = "tpush.keepalive"
    private const val MESSAGE_CHANNEL_ID = "tpush.messages"
    private const val MESSAGE_NOTIFICATION_BASE_ID = 3000
}

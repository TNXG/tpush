package moe.tnxg.push.core

import android.content.Context
import android.content.Intent
import android.os.Build

object Bridge {
    init {
        System.loadLibrary("tpush_core")
    }

    @JvmStatic
    fun init(context: Context) {
        val applicationContext = context.applicationContext
        val serverBaseUrl = Config.getServerBaseUrl(applicationContext)
        nativeInit(applicationContext, serverBaseUrl)
    }

    @JvmStatic
    fun startForegroundService(context: Context) {
        val intent = Intent(context, ForegroundService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            context.startForegroundService(intent)
        } else {
            context.startService(intent)
        }
    }

    @JvmStatic
    fun restartForegroundService(context: Context) {
        val intent = Intent(context, ForegroundService::class.java)
        context.stopService(intent)
        startForegroundService(context)
    }

    @JvmStatic
    external fun nativeInit(context: Context, serverBaseUrl: String): Boolean

    @JvmStatic
    external fun nativeGetDeviceId(): String

    @JvmStatic
    external fun nativeGetMessagesJson(): String

    @JvmStatic
    external fun nativeMarkRead(id: String)

    @JvmStatic
    external fun nativeDeleteMessage(id: String)

    @JvmStatic
    external fun nativeClearAll()

    @JvmStatic
    external fun nativeIngestRealtimeMessage(messageJson: String)
}

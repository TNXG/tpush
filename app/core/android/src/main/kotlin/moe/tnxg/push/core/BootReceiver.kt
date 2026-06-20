package moe.tnxg.push.core

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        Bridge.init(context)
        Bridge.startForegroundService(context)
    }
}


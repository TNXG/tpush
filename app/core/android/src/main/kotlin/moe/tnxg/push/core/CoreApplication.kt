package moe.tnxg.push.core

import android.app.Application

open class CoreApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Bridge.init(this)
        Bridge.startForegroundService(this)
    }
}


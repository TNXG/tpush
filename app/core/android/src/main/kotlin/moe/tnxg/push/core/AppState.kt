package moe.tnxg.push.core

import android.app.Activity
import android.app.Application
import android.content.Context
import android.os.Bundle

object AppState {
    private const val MODE_MULTI_PROCESS = 4
    private const val PREFERENCES_NAME = "tpush_app_state"
    private const val KEY_VISIBLE_ACTIVITY_COUNT = "visible_activity_count"
    private const val KEY_FRONTEND_VISIBLE = "frontend_visible"
    private const val KEY_LAST_FOREGROUND_AT = "last_foreground_at"
    private const val KEY_LAST_BACKGROUND_AT = "last_background_at"
    private var registered = false

    fun register(application: Application) {
        if (registered) {
            return
        }
        registered = true
        application.registerActivityLifecycleCallbacks(object : Application.ActivityLifecycleCallbacks {
            override fun onActivityStarted(activity: Activity) {
                val count = currentVisibleCount(application) + 1
                application.preferences()
                    .edit()
                    .putInt(KEY_VISIBLE_ACTIVITY_COUNT, count)
                    .putBoolean(KEY_FRONTEND_VISIBLE, true)
                    .putLong(KEY_LAST_FOREGROUND_AT, System.currentTimeMillis())
                    .apply()
            }

            override fun onActivityStopped(activity: Activity) {
                val count = (currentVisibleCount(application) - 1).coerceAtLeast(0)
                application.preferences()
                    .edit()
                    .putInt(KEY_VISIBLE_ACTIVITY_COUNT, count)
                    .putBoolean(KEY_FRONTEND_VISIBLE, count > 0)
                    .putLong(KEY_LAST_BACKGROUND_AT, System.currentTimeMillis())
                    .apply()
            }

            override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) = Unit
            override fun onActivityResumed(activity: Activity) = Unit
            override fun onActivityPaused(activity: Activity) = Unit
            override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) = Unit
            override fun onActivityDestroyed(activity: Activity) = Unit
        })
    }

    fun isFrontendVisible(context: Context): Boolean {
        return context.preferences().getBoolean(KEY_FRONTEND_VISIBLE, false) &&
            ResourceMetrics.snapshot(context).frontendPid > 0
    }

    fun snapshot(context: Context): Snapshot {
        val preferences = context.preferences()
        return Snapshot(
            frontendVisible = preferences.getBoolean(KEY_FRONTEND_VISIBLE, false),
            lastForegroundAt = preferences.getLong(KEY_LAST_FOREGROUND_AT, 0L),
            lastBackgroundAt = preferences.getLong(KEY_LAST_BACKGROUND_AT, 0L),
        )
    }

    private fun currentVisibleCount(context: Context): Int {
        return context.preferences().getInt(KEY_VISIBLE_ACTIVITY_COUNT, 0)
    }

    private fun Context.preferences() =
        getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)

    data class Snapshot(
        val frontendVisible: Boolean,
        val lastForegroundAt: Long,
        val lastBackgroundAt: Long,
    )
}

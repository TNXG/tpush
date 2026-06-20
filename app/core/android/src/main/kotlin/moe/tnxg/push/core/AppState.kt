package moe.tnxg.push.core

import android.app.Activity
import android.app.Application
import android.content.ComponentCallbacks2
import android.content.Context
import android.content.res.Configuration
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
        markBackground(application)
        application.registerActivityLifecycleCallbacks(object : Application.ActivityLifecycleCallbacks {
            override fun onActivityStarted(activity: Activity) {
                val count = currentVisibleCount(application) + 1
                markForeground(application, count)
            }

            override fun onActivityStopped(activity: Activity) {
                val count = (currentVisibleCount(application) - 1).coerceAtLeast(0)
                if (count > 0) {
                    markForeground(application, count)
                } else {
                    markBackground(application)
                }
            }

            override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) = Unit
            override fun onActivityResumed(activity: Activity) = Unit
            override fun onActivityPaused(activity: Activity) = Unit
            override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) = Unit
            override fun onActivityDestroyed(activity: Activity) = Unit
        })
        application.registerComponentCallbacks(object : ComponentCallbacks2 {
            override fun onTrimMemory(level: Int) {
                if (level >= ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN) {
                    markBackground(application)
                }
            }

            override fun onConfigurationChanged(newConfig: Configuration) = Unit
            override fun onLowMemory() = Unit
        })
    }

    fun isFrontendVisible(context: Context): Boolean {
        val preferences = context.preferences()
        val visibleActivityCount = preferences.getInt(KEY_VISIBLE_ACTIVITY_COUNT, 0)
        val frontendVisible = preferences.getBoolean(KEY_FRONTEND_VISIBLE, false)
        val lastForegroundAt = preferences.getLong(KEY_LAST_FOREGROUND_AT, 0L)
        val lastBackgroundAt = preferences.getLong(KEY_LAST_BACKGROUND_AT, 0L)
        return visibleActivityCount > 0 &&
            frontendVisible &&
            lastForegroundAt >= lastBackgroundAt &&
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

    private fun markForeground(context: Context, visibleActivityCount: Int) {
        context.preferences()
            .edit()
            .putInt(KEY_VISIBLE_ACTIVITY_COUNT, visibleActivityCount)
            .putBoolean(KEY_FRONTEND_VISIBLE, true)
            .putLong(KEY_LAST_FOREGROUND_AT, System.currentTimeMillis())
            .commit()
    }

    private fun markBackground(context: Context) {
        context.preferences()
            .edit()
            .putInt(KEY_VISIBLE_ACTIVITY_COUNT, 0)
            .putBoolean(KEY_FRONTEND_VISIBLE, false)
            .putLong(KEY_LAST_BACKGROUND_AT, System.currentTimeMillis())
            .commit()
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

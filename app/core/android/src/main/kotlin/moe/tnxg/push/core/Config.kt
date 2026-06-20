package moe.tnxg.push.core

import android.content.Context

object Config {
    private const val DEFAULT_SERVER_BASE_URL = "http://10.0.2.2:3000"
    private const val MODE_MULTI_PROCESS = 4
    private const val METADATA_SERVER_BASE_URL = "moe.tnxg.push.SERVER_BASE_URL"
    private const val PREFERENCES_NAME = "tpush_config"
    private const val KEY_SERVER_BASE_URL = "server_base_url"
    private const val KEY_CHANNEL = "channel"
    private const val KEY_CHANNEL_SECRET = "channel_secret"

    fun getServerBaseUrl(context: Context): String {
        val applicationContext = context.applicationContext
        val savedUrl = applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
            .getString(KEY_SERVER_BASE_URL, "")
            .orEmpty()
            .trim()

        if (savedUrl.isNotEmpty()) {
            return savedUrl
        }

        return applicationContext
            .packageManager
            .getApplicationInfo(applicationContext.packageName, 128)
            .metaData
            ?.getString(METADATA_SERVER_BASE_URL)
            ?: DEFAULT_SERVER_BASE_URL
    }

    fun setServerBaseUrl(context: Context, serverBaseUrl: String) {
        context.applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
            .edit()
            .putString(KEY_SERVER_BASE_URL, serverBaseUrl.trim().trimEnd('/'))
            .apply()
    }

    fun getChannel(context: Context): String {
        return context.applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
            .getString(KEY_CHANNEL, "default")
            .orEmpty()
            .trim()
            .ifEmpty { "default" }
    }

    fun getChannelSecret(context: Context): String {
        return context.applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
            .getString(KEY_CHANNEL_SECRET, "")
            .orEmpty()
    }

    fun setChannelConfig(context: Context, channel: String, channelSecret: String) {
        context.applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
            .edit()
            .putString(KEY_CHANNEL, channel.trim().ifEmpty { "default" })
            .putString(KEY_CHANNEL_SECRET, channelSecret.trim())
            .apply()
    }
}

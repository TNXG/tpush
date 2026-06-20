package moe.tnxg.push.core

import android.content.Context

object ConnectionStatus {
    private const val MODE_MULTI_PROCESS = 4
    private const val PREFERENCES_NAME = "tpush_connection_status"
    private const val KEY_CONNECTED = "connected"
    private const val KEY_STATE = "state"
    private const val KEY_SERVER_BASE_URL = "server_base_url"
    private const val KEY_LAST_CONNECTED_AT = "last_connected_at"
    private const val KEY_LAST_MESSAGE_AT = "last_message_at"
    private const val KEY_LAST_ERROR = "last_error"
    private const val KEY_REGISTER_STATE = "register_state"
    private const val KEY_REGISTER_LAST_AT = "register_last_at"
    private const val KEY_REGISTER_LAST_ERROR = "register_last_error"
    private const val KEY_SYNC_STATE = "sync_state"
    private const val KEY_SYNC_LAST_AT = "sync_last_at"
    private const val KEY_SYNC_LAST_COUNT = "sync_last_count"
    private const val KEY_SYNC_LAST_ERROR = "sync_last_error"
    private const val KEY_WS_STATE = "ws_state"
    private const val KEY_WS_LAST_CONNECTED_AT = "ws_last_connected_at"
    private const val KEY_WS_LAST_MESSAGE_AT = "ws_last_message_at"
    private const val KEY_WS_LAST_ERROR = "ws_last_error"
    private const val KEY_DECRYPT_STATE = "decrypt_state"
    private const val KEY_DECRYPT_LAST_ERROR = "decrypt_last_error"
    private const val KEY_SERVICE_RUNNING = "service_running"
    private const val KEY_SERVICE_STARTED_AT = "service_started_at"
    private const val KEY_SERVICE_STOPPED_AT = "service_stopped_at"

    fun setConnecting(context: Context, serverBaseUrl: String) {
        context.preferences()
            .edit()
            .putBoolean(KEY_CONNECTED, false)
            .putString(KEY_STATE, "connecting")
            .putString(KEY_WS_STATE, "connecting")
            .putString(KEY_SERVER_BASE_URL, serverBaseUrl)
            .remove(KEY_LAST_ERROR)
            .remove(KEY_WS_LAST_ERROR)
            .apply()
    }

    fun setConnected(context: Context, serverBaseUrl: String) {
        context.preferences()
            .edit()
            .putBoolean(KEY_CONNECTED, true)
            .putString(KEY_STATE, "connected")
            .putString(KEY_WS_STATE, "connected")
            .putString(KEY_SERVER_BASE_URL, serverBaseUrl)
            .putLong(KEY_LAST_CONNECTED_AT, System.currentTimeMillis())
            .putLong(KEY_WS_LAST_CONNECTED_AT, System.currentTimeMillis())
            .remove(KEY_LAST_ERROR)
            .remove(KEY_WS_LAST_ERROR)
            .apply()
    }

    fun setMessageReceived(context: Context) {
        context.preferences()
            .edit()
            .putLong(KEY_LAST_MESSAGE_AT, System.currentTimeMillis())
            .putLong(KEY_WS_LAST_MESSAGE_AT, System.currentTimeMillis())
            .apply()
    }

    fun setDisconnected(context: Context, error: String?) {
        context.preferences()
            .edit()
            .putBoolean(KEY_CONNECTED, false)
            .putString(KEY_STATE, "reconnecting")
            .putString(KEY_WS_STATE, "reconnecting")
            .putString(KEY_LAST_ERROR, error ?: "")
            .putString(KEY_WS_LAST_ERROR, error ?: "")
            .apply()
    }

    fun setRegistering(context: Context) {
        context.preferences().edit().putString(KEY_REGISTER_STATE, "registering").apply()
    }

    fun setRegisterSuccess(context: Context) {
        context.preferences()
            .edit()
            .putString(KEY_REGISTER_STATE, "ok")
            .putLong(KEY_REGISTER_LAST_AT, System.currentTimeMillis())
            .remove(KEY_REGISTER_LAST_ERROR)
            .apply()
    }

    fun setRegisterFailed(context: Context, error: String?) {
        context.preferences()
            .edit()
            .putString(KEY_REGISTER_STATE, "failed")
            .putString(KEY_REGISTER_LAST_ERROR, error ?: "")
            .apply()
    }

    fun setSyncing(context: Context) {
        context.preferences().edit().putString(KEY_SYNC_STATE, "syncing").apply()
    }

    fun setSyncSuccess(context: Context, count: Int) {
        context.preferences()
            .edit()
            .putString(KEY_SYNC_STATE, "ok")
            .putLong(KEY_SYNC_LAST_AT, System.currentTimeMillis())
            .putInt(KEY_SYNC_LAST_COUNT, count)
            .remove(KEY_SYNC_LAST_ERROR)
            .apply()
    }

    fun setSyncFailed(context: Context, error: String?) {
        context.preferences()
            .edit()
            .putString(KEY_SYNC_STATE, "failed")
            .putString(KEY_SYNC_LAST_ERROR, error ?: "")
            .apply()
    }

    fun setDecryptOk(context: Context) {
        context.preferences()
            .edit()
            .putString(KEY_DECRYPT_STATE, "ok")
            .remove(KEY_DECRYPT_LAST_ERROR)
            .apply()
    }

    fun setDecryptFailed(context: Context, error: String?) {
        context.preferences()
            .edit()
            .putString(KEY_DECRYPT_STATE, "failed")
            .putString(KEY_DECRYPT_LAST_ERROR, error ?: "")
            .apply()
    }

    fun setServiceRunning(context: Context, running: Boolean) {
        val editor = context.preferences().edit().putBoolean(KEY_SERVICE_RUNNING, running)
        if (running) {
            editor.putLong(KEY_SERVICE_STARTED_AT, System.currentTimeMillis())
        } else {
            editor.putLong(KEY_SERVICE_STOPPED_AT, System.currentTimeMillis())
        }
        editor.apply()
    }

    fun snapshot(context: Context): Snapshot {
        val preferences = context.preferences()
        return Snapshot(
            connected = preferences.getBoolean(KEY_CONNECTED, false),
            state = preferences.getString(KEY_STATE, "idle") ?: "idle",
            serverBaseUrl = preferences.getString(KEY_SERVER_BASE_URL, "") ?: "",
            lastConnectedAt = preferences.getLong(KEY_LAST_CONNECTED_AT, 0L),
            lastMessageAt = preferences.getLong(KEY_LAST_MESSAGE_AT, 0L),
            lastError = preferences.getString(KEY_LAST_ERROR, "") ?: "",
            registerState = preferences.getString(KEY_REGISTER_STATE, "idle") ?: "idle",
            registerLastAt = preferences.getLong(KEY_REGISTER_LAST_AT, 0L),
            registerLastError = preferences.getString(KEY_REGISTER_LAST_ERROR, "") ?: "",
            syncState = preferences.getString(KEY_SYNC_STATE, "idle") ?: "idle",
            syncLastAt = preferences.getLong(KEY_SYNC_LAST_AT, 0L),
            syncLastCount = preferences.getInt(KEY_SYNC_LAST_COUNT, 0),
            syncLastError = preferences.getString(KEY_SYNC_LAST_ERROR, "") ?: "",
            wsState = preferences.getString(KEY_WS_STATE, "idle") ?: "idle",
            wsLastConnectedAt = preferences.getLong(KEY_WS_LAST_CONNECTED_AT, 0L),
            wsLastMessageAt = preferences.getLong(KEY_WS_LAST_MESSAGE_AT, 0L),
            wsLastError = preferences.getString(KEY_WS_LAST_ERROR, "") ?: "",
            decryptState = preferences.getString(KEY_DECRYPT_STATE, "idle") ?: "idle",
            decryptLastError = preferences.getString(KEY_DECRYPT_LAST_ERROR, "") ?: "",
            serviceRunning = preferences.getBoolean(KEY_SERVICE_RUNNING, false),
            serviceStartedAt = preferences.getLong(KEY_SERVICE_STARTED_AT, 0L),
            serviceStoppedAt = preferences.getLong(KEY_SERVICE_STOPPED_AT, 0L),
        )
    }

    data class Snapshot(
        val connected: Boolean,
        val state: String,
        val serverBaseUrl: String,
        val lastConnectedAt: Long,
        val lastMessageAt: Long,
        val lastError: String,
        val registerState: String,
        val registerLastAt: Long,
        val registerLastError: String,
        val syncState: String,
        val syncLastAt: Long,
        val syncLastCount: Int,
        val syncLastError: String,
        val wsState: String,
        val wsLastConnectedAt: Long,
        val wsLastMessageAt: Long,
        val wsLastError: String,
        val decryptState: String,
        val decryptLastError: String,
        val serviceRunning: Boolean,
        val serviceStartedAt: Long,
        val serviceStoppedAt: Long,
    )

    private fun Context.preferences() =
        getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE or MODE_MULTI_PROCESS)
}

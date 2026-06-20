package moe.tnxg.push.core

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod
import com.facebook.react.bridge.WritableArray
import com.facebook.react.bridge.WritableNativeArray
import com.facebook.react.bridge.WritableNativeMap
import org.json.JSONArray

class TpushModule(private val reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {
    override fun getName(): String = "TPush"

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getMessages(): WritableArray {
        Bridge.init(reactContext)
        val messages = JSONArray(Bridge.nativeGetMessagesJson())
        val result = WritableNativeArray()
        for (index in 0 until messages.length()) {
            val message = messages.getJSONObject(index)
            val item = WritableNativeMap()
            item.putString("id", message.optString("id"))
            item.putString("title", message.optString("title"))
            item.putString("content", message.optString("content"))
            item.putString("payload", message.optString("payload"))
            item.putString("kind", message.optString("kind"))
            item.putString("received_at", message.optString("received_at"))
            item.putBoolean("read", message.optBoolean("read"))
            result.pushMap(item)
        }
        return result
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun markRead(id: String) {
        Bridge.init(reactContext)
        Bridge.nativeMarkRead(id)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getDeviceId(): String {
        Bridge.init(reactContext)
        return Bridge.nativeGetDeviceId()
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getServerBaseUrl(): String {
        return Config.getServerBaseUrl(reactContext)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun setServerBaseUrl(serverBaseUrl: String) {
        Config.setServerBaseUrl(reactContext, serverBaseUrl)
        Bridge.init(reactContext)
        Bridge.restartForegroundService(reactContext)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getChannel(): String {
        return Config.getChannel(reactContext)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getChannelSecret(): String {
        return Config.getChannelSecret(reactContext)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun setChannelConfig(channel: String, channelSecret: String) {
        Config.setChannelConfig(reactContext, channel, channelSecret)
        Bridge.init(reactContext)
        Bridge.restartForegroundService(reactContext)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun deleteMessage(id: String) {
        Bridge.init(reactContext)
        Bridge.nativeDeleteMessage(id)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun clearAll() {
        Bridge.init(reactContext)
        Bridge.nativeClearAll()
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun getConnectionStatusJson(): String {
        val status = ConnectionStatus.snapshot(reactContext)
        val appState = AppState.snapshot(reactContext)
        val metrics = ResourceMetrics.snapshot(reactContext)
        return org.json.JSONObject()
            .put("connected", status.connected)
            .put("state", status.state)
            .put("serverBaseUrl", status.serverBaseUrl)
            .put("channel", Config.getChannel(reactContext))
            .put("hasChannelSecret", Config.getChannelSecret(reactContext).isNotBlank())
            .put("lastConnectedAt", status.lastConnectedAt)
            .put("lastMessageAt", status.lastMessageAt)
            .put("lastError", status.lastError)
            .put("registerState", status.registerState)
            .put("registerLastAt", status.registerLastAt)
            .put("registerLastError", status.registerLastError)
            .put("syncState", status.syncState)
            .put("syncLastAt", status.syncLastAt)
            .put("syncLastCount", status.syncLastCount)
            .put("syncLastError", status.syncLastError)
            .put("wsState", status.wsState)
            .put("wsLastConnectedAt", status.wsLastConnectedAt)
            .put("wsLastMessageAt", status.wsLastMessageAt)
            .put("wsLastError", status.wsLastError)
            .put("decryptState", status.decryptState)
            .put("decryptLastError", status.decryptLastError)
            .put("serviceRunning", status.serviceRunning && metrics.serviceRunning)
            .put("serviceStartedAt", status.serviceStartedAt)
            .put("serviceStoppedAt", status.serviceStoppedAt)
            .put("frontendVisible", appState.frontendVisible)
            .put("lastForegroundAt", appState.lastForegroundAt)
            .put("lastBackgroundAt", appState.lastBackgroundAt)
            .put("corePssKb", metrics.corePssKb)
            .put("frontendPssKb", metrics.frontendPssKb)
            .put("coreNativeHeapKb", metrics.coreNativeHeapKb)
            .put("frontendNativeHeapKb", metrics.frontendNativeHeapKb)
            .put("servicePid", metrics.servicePid)
            .put("frontendPid", metrics.frontendPid)
            .toString()
    }
}

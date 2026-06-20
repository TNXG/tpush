package moe.tnxg.push.core

import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import okhttp3.Call
import okhttp3.Callback
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONArray
import org.json.JSONObject
import android.util.Log
import java.io.IOException
import java.util.concurrent.TimeUnit

class ForegroundService : Service() {
    private val mainHandler = Handler(Looper.getMainLooper())
    private val httpClient = OkHttpClient.Builder()
        .pingInterval(25, TimeUnit.SECONDS)
        .retryOnConnectionFailure(true)
        .build()
    private var webSocket: WebSocket? = null
    private var reconnectDelayMs = INITIAL_RECONNECT_DELAY_MS
    private lateinit var serverBaseUrl: String
    private lateinit var deviceId: String
    private lateinit var channelName: String
    private lateinit var channelSecret: String
    private lateinit var deviceInfo: DeviceInfo

    override fun onCreate() {
        super.onCreate()
        channelName = Config.getChannel(this)
        startForeground(NOTIFICATION_ID, Notifications.buildForegroundNotification(this, channelName))
        Bridge.init(this)
        ConnectionStatus.setServiceRunning(this, true)
        serverBaseUrl = readServerBaseUrl()
        deviceId = Bridge.nativeGetDeviceId()
        channelSecret = Config.getChannelSecret(this)
        deviceInfo = DeviceInfo.fromService(this)
        ConnectionStatus.setConnecting(this, serverBaseUrl)
        registerDevice()
        syncMessages()
        connectWebSocket()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        Bridge.startForegroundService(this)
        super.onTaskRemoved(rootIntent)
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        ConnectionStatus.setServiceRunning(this, false)
        webSocket?.close(1000, "service stopped")
        mainHandler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }

    private fun readServerBaseUrl(): String {
        return Config.getServerBaseUrl(this)
    }

    private fun registerDevice() {
        val body = JSONObject()
            .put("deviceId", deviceId)
            .put("channel", channelName)
            .put("device", deviceInfo.toJson())
            .put("auth", Crypto.authJson(channelName, channelSecret, deviceId))
            .toString()
            .toRequestBody(JSON_MEDIA_TYPE)
        val request = Request.Builder()
            .url("${serverBaseUrl.trimEnd('/')}/api/devices/register")
            .post(body)
            .build()
        ConnectionStatus.setRegistering(this)
        httpClient.newCall(request).enqueue(object : Callback {
            override fun onFailure(call: Call, exception: IOException) {
                ConnectionStatus.setRegisterFailed(this@ForegroundService, exception.message)
                ConnectionStatus.setDisconnected(this@ForegroundService, exception.message)
            }

            override fun onResponse(call: Call, response: Response) {
                response.use {
                    if (it.isSuccessful) {
                        ConnectionStatus.setRegisterSuccess(this@ForegroundService)
                    } else {
                        Log.e(
                            LOG_TAG,
                            "[AUTH_FAIL_REGISTER] channel=$channelName reason=HTTP_${it.code} keyPresent=${channelSecret.isNotBlank()} keyCheckStatus=failed",
                        )
                        ConnectionStatus.setRegisterFailed(
                            this@ForegroundService,
                            "HTTP ${it.code}",
                        )
                    }
                }
            }
        })
    }

    private fun syncMessages() {
        val request = Request.Builder()
            .url("${serverBaseUrl.trimEnd('/')}/api/messages/sync?${channelQuery()}")
            .get()
            .build()
        ConnectionStatus.setSyncing(this)
        httpClient.newCall(request).enqueue(object : Callback {
            override fun onFailure(call: Call, exception: IOException) {
                ConnectionStatus.setSyncFailed(this@ForegroundService, exception.message)
                ConnectionStatus.setDisconnected(this@ForegroundService, exception.message)
            }

            override fun onResponse(call: Call, response: Response) {
                response.use {
                    if (!it.isSuccessful) {
                        Log.e(
                            LOG_TAG,
                            "[AUTH_FAIL_SYNC] channel=$channelName reason=HTTP_${it.code} keyPresent=${channelSecret.isNotBlank()} keyCheckStatus=failed",
                        )
                        ConnectionStatus.setSyncFailed(
                            this@ForegroundService,
                            "HTTP ${it.code}",
                        )
                        return
                    }
                    val responseBody = it.body?.string() ?: return
                    val messages = JSONArray(responseBody)
                    for (index in 0 until messages.length()) {
                        try {
                            val envelope = messages.getJSONObject(index)
                            if (envelope.optBoolean("encrypted", false)) {
                                val plaintext = Crypto.decryptEnvelope(envelope, channelName, channelSecret)
                                if (plaintext == null) {
                                    Log.e(
                                        LOG_TAG,
                                        "[DECRYPT_FAIL_SYNC] channel=$channelName reason=key_mismatch_or_missing keyPresent=${channelSecret.isNotBlank()} keyCheckStatus=failed",
                                    )
                                    ConnectionStatus.setDecryptFailed(
                                        this@ForegroundService,
                                        "同步解密失败：密钥不正确或缺失，频道=$channelName",
                                    )
                                    continue
                                }
                                Bridge.nativeIngestRealtimeMessage(plaintext)
                                ConnectionStatus.setDecryptOk(this@ForegroundService)
                            } else {
                                val msgData = envelope.optString("data", "")
                                if (msgData.isNotEmpty()) {
                                    Bridge.nativeIngestRealtimeMessage(msgData)
                                } else {
                                    Bridge.nativeIngestRealtimeMessage(envelope.toString())
                                }
                            }
                        } catch (e: Exception) {
                            Bridge.nativeIngestRealtimeMessage(messages.getJSONObject(index).toString())
                        }
                    }
                    ConnectionStatus.setSyncSuccess(this@ForegroundService, messages.length())
                }
            }
        })
    }

    private fun connectWebSocket() {
        val request = Request.Builder()
            .url(buildWebSocketUrl())
            .build()
        webSocket = httpClient.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                reconnectDelayMs = INITIAL_RECONNECT_DELAY_MS
                ConnectionStatus.setConnected(this@ForegroundService, serverBaseUrl)
                response.close()
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                var finalMessageText = text
                try {
                    val envelope = JSONObject(text)
                    if (envelope.optBoolean("encrypted", false)) {
                        val plaintext = Crypto.decryptEnvelope(envelope, channelName, channelSecret)
                        if (plaintext == null) {
                            Log.e(
                                LOG_TAG,
                                "[DECRYPT_FAIL_WS] channel=$channelName reason=key_mismatch_or_missing keyPresent=${channelSecret.isNotBlank()} keyCheckStatus=failed dataLen=${envelope.optString("data").length}",
                            )
                            ConnectionStatus.setDecryptFailed(this@ForegroundService, "解密失败：密钥不正确或缺失，频道=$channelName")
                            webSocket.close(4001, "decrypt_failed")
                            return
                        }
                        finalMessageText = plaintext
                        Bridge.nativeIngestRealtimeMessage(plaintext)
                        ConnectionStatus.setDecryptOk(this@ForegroundService)
                    } else {
                        val msgData = envelope.optString("data", "")
                        if (msgData.isNotEmpty()) {
                            finalMessageText = msgData
                        }
                        Bridge.nativeIngestRealtimeMessage(finalMessageText)
                    }
                } catch (e: Exception) {
                    Bridge.nativeIngestRealtimeMessage(text)
                }
                ConnectionStatus.setMessageReceived(this@ForegroundService)
                if (!AppState.isFrontendVisible(this@ForegroundService)) {
                    Notifications.showPushNotification(this@ForegroundService, finalMessageText)
                }
            }

            override fun onFailure(webSocket: WebSocket, throwable: Throwable, response: Response?) {
                Log.e(
                    LOG_TAG,
                    "[AUTH_OR_WS_FAIL] channel=$channelName reason=${throwable.message ?: "unknown"} httpCode=${response?.code ?: -1} keyPresent=${channelSecret.isNotBlank()} keyCheckStatus=${if (response?.code == 401) "failed" else "unknown"}",
                )
                response?.close()
                ConnectionStatus.setDisconnected(this@ForegroundService, throwable.message)
                scheduleReconnect()
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                ConnectionStatus.setDisconnected(this@ForegroundService, reason)
                if (code != 4001) {
                    scheduleReconnect()
                }
            }
        })
    }

    private fun scheduleReconnect() {
        mainHandler.postDelayed({
            syncMessages()
            connectWebSocket()
        }, reconnectDelayMs)
        reconnectDelayMs = (reconnectDelayMs * 2).coerceAtMost(MAX_RECONNECT_DELAY_MS)
    }

    private fun channelQuery(): String {
        val parts = mutableListOf(
            "deviceId=${Crypto.encodeUrl(deviceId)}",
            "channel=${Crypto.encodeUrl(channelName)}",
        )
        parts += Crypto.authQueryParts(channelName, channelSecret, deviceId)
        return parts.joinToString("&")
    }

    private fun buildWebSocketUrl(): String {
        val authQuery = Crypto.authQuery(channelName, channelSecret, "ws")
        val baseUrl = "${serverBaseUrl.toWebSocketBaseUrl()}/api/channels/${Crypto.encodeUrl(channelName)}/stream"
        val deviceQuery = deviceInfo.toQueryParts(deviceId).joinToString("&")
        return listOf(deviceQuery, authQuery).filter { part -> part.isNotBlank() }.joinToString("&").let { query ->
            if (query.isBlank()) baseUrl else "$baseUrl?$query"
        }
    }

    private companion object {
        const val INITIAL_RECONNECT_DELAY_MS = 1_000L
        const val MAX_RECONNECT_DELAY_MS = 60_000L
        const val NOTIFICATION_ID = 2001
        const val LOG_TAG = "TPush"
        val JSON_MEDIA_TYPE = "application/json; charset=utf-8".toMediaType()
    }
}

private data class DeviceInfo(
    val deviceName: String,
    val systemName: String,
    val systemVersion: String,
    val appVersion: String,
) {
    fun toJson(): JSONObject {
        return JSONObject()
            .put("deviceName", deviceName)
            .put("systemName", systemName)
            .put("systemVersion", systemVersion)
            .put("appVersion", appVersion)
    }

    fun toQueryParts(deviceId: String): List<String> {
        return listOf(
            "deviceId=${Crypto.encodeUrl(deviceId)}",
            "deviceName=${Crypto.encodeUrl(deviceName)}",
            "systemName=${Crypto.encodeUrl(systemName)}",
            "systemVersion=${Crypto.encodeUrl(systemVersion)}",
            "appVersion=${Crypto.encodeUrl(appVersion)}",
        )
    }

    companion object {
        fun fromService(service: Service): DeviceInfo {
            val packageInfo = service.packageManager.getPackageInfoCompat(service.packageName)
            val applicationLabel = service.packageManager
                .getApplicationLabel(service.applicationInfo)
                .toString()
            return DeviceInfo(
                deviceName = listOf(Build.MANUFACTURER, Build.MODEL)
                    .filter { value -> value.isNotBlank() }
                    .joinToString(" ")
                    .ifBlank { Build.DEVICE },
                systemName = "Android",
                systemVersion = "${Build.VERSION.RELEASE} (SDK ${Build.VERSION.SDK_INT})",
                appVersion = "$applicationLabel ${packageInfo.versionName ?: "unknown"}",
            )
        }
    }
}

private fun String.toWebSocketBaseUrl(): String {
    val trimmedUrl = trimEnd('/')
    return when {
        trimmedUrl.startsWith("https://") -> "wss://${trimmedUrl.removePrefix("https://")}"
        trimmedUrl.startsWith("http://") -> "ws://${trimmedUrl.removePrefix("http://")}"
        else -> trimmedUrl
    }
}

@Suppress("DEPRECATION")
private fun android.content.pm.PackageManager.getPackageInfoCompat(packageName: String): android.content.pm.PackageInfo {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getPackageInfo(packageName, android.content.pm.PackageManager.PackageInfoFlags.of(0))
    } else {
        getPackageInfo(packageName, 0)
    }
}

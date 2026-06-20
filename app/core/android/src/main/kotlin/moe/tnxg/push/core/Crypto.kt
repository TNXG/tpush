package moe.tnxg.push.core

import android.util.Base64
import android.util.Log
import org.json.JSONObject
import java.net.URLEncoder
import java.security.MessageDigest
import java.util.UUID
import javax.crypto.Cipher
import javax.crypto.Mac
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

object Crypto {
    fun authQuery(channelName: String, channelSecret: String, subject: String): String {
        return authQueryParts(channelName, channelSecret, subject).joinToString("&")
    }

    fun authQueryParts(channelName: String, channelSecret: String, subject: String): List<String> {
        if (channelSecret.isBlank()) {
            return emptyList()
        }

        val timestamp = System.currentTimeMillis().toString()
        val nonce = UUID.randomUUID().toString()
        val signature = signChannelPayload(channelName, channelSecret, subject, timestamp, nonce)
        return listOf(
            "ts=${encodeUrl(timestamp)}",
            "nonce=${encodeUrl(nonce)}",
            "signature=${encodeUrl(signature)}",
        )
    }

    fun authJson(channelName: String, channelSecret: String, subject: String): JSONObject {
        if (channelSecret.isBlank()) {
            return JSONObject()
        }

        val timestamp = System.currentTimeMillis().toString()
        val nonce = UUID.randomUUID().toString()
        return JSONObject()
            .put("ts", timestamp)
            .put("nonce", nonce)
            .put("signature", signChannelPayload(channelName, channelSecret, subject, timestamp, nonce))
    }

    fun decryptEnvelope(envelope: JSONObject, channelName: String, channelSecret: String): String? {
        if (channelSecret.isBlank()) {
            Log.e(
                LOG_TAG,
                "[DECRYPT_REJECT] channel=$channelName reason=encrypted_public_channel keyPresent=false keyCheckStatus=failed",
            )
            return null
        }

        return try {
            val envelopeChannel = envelope.optString("channel")
            if (envelopeChannel != channelName) {
                Log.e(
                    LOG_TAG,
                    "[DECRYPT_REJECT] channel=$channelName envelopeChannel=$envelopeChannel reason=channel_mismatch keyCheckStatus=failed",
                )
                return null
            }

            val payload = Base64.decode(envelope.getString("data"), Base64.DEFAULT)
            if (payload.size < NONCE_SIZE_BYTES) {
                Log.e(
                    LOG_TAG,
                    "[DECRYPT_REJECT] channel=$channelName reason=payload_too_short keyCheckStatus=failed",
                )
                return null
            }

            val nonce = payload.copyOfRange(0, NONCE_SIZE_BYTES)
            val ciphertext = payload.copyOfRange(NONCE_SIZE_BYTES, payload.size)
            val keyBytes = MessageDigest.getInstance("SHA-256").digest(channelSecret.toByteArray())
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.DECRYPT_MODE, SecretKeySpec(keyBytes, "AES"), GCMParameterSpec(128, nonce))
            cipher.updateAAD(channelName.toByteArray(Charsets.UTF_8))
            String(cipher.doFinal(ciphertext), Charsets.UTF_8)
        } catch (exception: Exception) {
            Log.e(
                LOG_TAG,
                "[DECRYPT_EXCEPTION] channel=$channelName reason=${exception.javaClass.simpleName} keyPresent=true keyCheckStatus=failed",
                exception,
            )
            null
        }
    }

    fun encodeUrl(value: String): String = URLEncoder.encode(value, Charsets.UTF_8.name())

    private fun signChannelPayload(
        channelName: String,
        channelSecret: String,
        subject: String,
        timestamp: String,
        nonce: String,
    ): String {
        val payload = "$channelName:$subject:$timestamp:$nonce"
        val mac = Mac.getInstance("HmacSHA256")
        mac.init(SecretKeySpec(channelSecret.toByteArray(Charsets.UTF_8), "HmacSHA256"))
        return mac.doFinal(payload.toByteArray(Charsets.UTF_8)).joinToString("") { byte ->
            "%02x".format(byte)
        }
    }

    private const val NONCE_SIZE_BYTES = 12
    private const val LOG_TAG = "TPush"
}

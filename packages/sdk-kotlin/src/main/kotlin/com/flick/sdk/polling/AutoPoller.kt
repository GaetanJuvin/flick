package com.flick.sdk.polling

import com.flick.sdk.model.ApiResponse
import com.flick.sdk.model.FullFlagConfig
import com.flick.sdk.network.HttpClient
import kotlinx.coroutines.*
import kotlinx.serialization.json.Json
import kotlin.math.min
import kotlin.math.pow

class AutoPoller(
    private val httpClient: HttpClient,
    private val intervalSeconds: Long,
    private val onData: (FullFlagConfig) -> Unit,
    private val onError: (Exception) -> Unit,
) {
    private var job: Job? = null
    private var etag: String? = null
    private var failureCount = 0
    private val maxBackoffSeconds = 60L
    private val json = Json { ignoreUnknownKeys = true }

    fun start(scope: CoroutineScope) {
        job = scope.launch {
            poll() // Initial poll
            while (isActive) {
                val delay = if (failureCount > 0) {
                    min(intervalSeconds * 2.0.pow(failureCount).toLong(), maxBackoffSeconds)
                } else {
                    intervalSeconds
                }
                delay(delay * 1000)
                poll()
            }
        }
    }

    fun stop() {
        job?.cancel()
        job = null
    }

    private suspend fun poll() {
        try {
            val response = httpClient.get("/evaluate/config", etag)

            if (response.statusCode == 304) {
                failureCount = 0
                return
            }

            if (response.statusCode != 200) {
                throw Exception("Polling failed: ${response.statusCode}")
            }

            response.etag?.let { etag = it }

            val apiResponse = json.decodeFromString<ApiResponse<FullFlagConfig>>(response.body)
            failureCount = 0
            onData(apiResponse.data)
        } catch (e: Exception) {
            failureCount++
            onError(e)
        }
    }
}

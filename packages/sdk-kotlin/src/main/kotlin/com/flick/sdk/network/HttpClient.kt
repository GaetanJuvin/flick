package com.flick.sdk.network

import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

data class HttpResponse(
    val statusCode: Int,
    val body: String,
    val etag: String?,
)

class HttpClient(
    private val baseUrl: String,
    private val apiKey: String,
) {
    private val client = OkHttpClient.Builder()
        .connectTimeout(10, TimeUnit.SECONDS)
        .readTimeout(10, TimeUnit.SECONDS)
        .build()

    fun get(path: String, ifNoneMatch: String? = null): HttpResponse {
        val request = Request.Builder()
            .url("${baseUrl.trimEnd('/')}$path")
            .addHeader("Authorization", "Bearer $apiKey")
            .addHeader("Accept", "application/json")
            .apply {
                if (ifNoneMatch != null) {
                    addHeader("If-None-Match", ifNoneMatch)
                }
            }
            .build()

        client.newCall(request).execute().use { response ->
            return HttpResponse(
                statusCode = response.code,
                body = response.body?.string() ?: "",
                etag = response.header("ETag"),
            )
        }
    }

    fun close() {
        client.dispatcher.executorService.shutdown()
        client.connectionPool.evictAll()
    }
}

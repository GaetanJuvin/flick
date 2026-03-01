package com.flick.sdk

data class FlickConfig(
    val serverUrl: String,
    val apiKey: String,
    val pollingIntervalSeconds: Long = 30,
    val defaultValues: Map<String, Boolean> = emptyMap(),
)

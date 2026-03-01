package com.flick.sdk

data class FlickContext(
    val key: String,
    val attributes: Map<String, Any> = emptyMap(),
) {
    companion object {
        fun create(key: String, attributes: Map<String, Any> = emptyMap()): FlickContext {
            return FlickContext(key, attributes)
        }
    }
}

package com.flick.sdk.model

data class EvaluationResult(
    val flagKey: String,
    val enabled: Boolean,
    val gateType: String,
    val reason: String,
)

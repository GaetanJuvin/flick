package com.flick.sdk.model

import kotlinx.serialization.Serializable

@Serializable
data class GroupRule(
    val attribute: String,
    val operator: String,
    val value: kotlinx.serialization.json.JsonElement,
)

@Serializable
data class FlagGroup(
    val id: String,
    val rules: List<GroupRule>,
)

@Serializable
data class FlagConfig(
    val key: String,
    val gate_type: String,
    val enabled: Boolean,
    val gate_config: Map<String, kotlinx.serialization.json.JsonElement> = emptyMap(),
    val groups: List<FlagGroup> = emptyList(),
)

@Serializable
data class FullFlagConfig(
    val environment: String,
    val flags: List<FlagConfig>,
    val version: String,
)

@Serializable
data class ApiResponse<T>(
    val data: T,
)

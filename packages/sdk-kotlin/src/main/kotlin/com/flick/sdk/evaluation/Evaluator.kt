package com.flick.sdk.evaluation

import com.flick.sdk.FlickContext
import com.flick.sdk.model.EvaluationResult
import com.flick.sdk.model.FlagConfig
import com.flick.sdk.model.GroupRule
import kotlinx.serialization.json.*

object Evaluator {

    fun evaluate(flag: FlagConfig?, context: FlickContext): EvaluationResult {
        if (flag == null) {
            return EvaluationResult("", false, "boolean", "flag_not_found")
        }

        if (!flag.enabled) {
            return EvaluationResult(flag.key, false, flag.gate_type, "flag_disabled")
        }

        return when (flag.gate_type) {
            "boolean" -> EvaluationResult(flag.key, true, "boolean", "boolean_on")
            "percentage" -> evaluatePercentage(flag, context)
            "group" -> evaluateGroup(flag, context)
            else -> EvaluationResult(flag.key, false, flag.gate_type, "flag_not_found")
        }
    }

    private fun evaluatePercentage(flag: FlagConfig, context: FlickContext): EvaluationResult {
        val percentage = flag.gate_config["percentage"]?.jsonPrimitive?.intOrNull ?: 0
        val hash = murmurhash3("${flag.key}${context.key}") % 100
        val bucket = if (hash < 0) hash + 100 else hash
        val enabled = bucket < percentage
        return EvaluationResult(
            flag.key, enabled, "percentage",
            if (enabled) "percentage_match" else "percentage_miss"
        )
    }

    private fun evaluateGroup(flag: FlagConfig, context: FlickContext): EvaluationResult {
        if (flag.groups.isEmpty()) {
            return EvaluationResult(flag.key, false, "group", "group_miss")
        }

        val matched = flag.groups.any { group ->
            group.rules.all { rule -> matchRule(rule, context.attributes) }
        }

        return EvaluationResult(
            flag.key, matched, "group",
            if (matched) "group_match" else "group_miss"
        )
    }

    private fun matchRule(rule: GroupRule, attributes: Map<String, Any>): Boolean {
        val attrValue = attributes[rule.attribute] ?: return false

        return when (rule.operator) {
            "eq" -> attrValue.toString() == extractStringValue(rule.value)
            "neq" -> attrValue.toString() != extractStringValue(rule.value)
            "in" -> extractListValue(rule.value).contains(attrValue.toString())
            "not_in" -> !extractListValue(rule.value).contains(attrValue.toString())
            "contains" -> attrValue.toString().contains(extractStringValue(rule.value))
            "starts_with" -> attrValue.toString().startsWith(extractStringValue(rule.value))
            "ends_with" -> attrValue.toString().endsWith(extractStringValue(rule.value))
            "gt" -> (attrValue as? Number)?.toDouble()?.let { it > extractNumberValue(rule.value) } ?: false
            "gte" -> (attrValue as? Number)?.toDouble()?.let { it >= extractNumberValue(rule.value) } ?: false
            "lt" -> (attrValue as? Number)?.toDouble()?.let { it < extractNumberValue(rule.value) } ?: false
            "lte" -> (attrValue as? Number)?.toDouble()?.let { it <= extractNumberValue(rule.value) } ?: false
            "regex" -> try { Regex(extractStringValue(rule.value)).containsMatchIn(attrValue.toString()) } catch (_: Exception) { false }
            else -> false
        }
    }

    private fun extractStringValue(value: JsonElement): String = when (value) {
        is JsonPrimitive -> value.content
        else -> value.toString()
    }

    private fun extractListValue(value: JsonElement): List<String> = when (value) {
        is JsonArray -> value.map { it.jsonPrimitive.content }
        else -> listOf(value.toString())
    }

    private fun extractNumberValue(value: JsonElement): Double = when (value) {
        is JsonPrimitive -> value.doubleOrNull ?: 0.0
        else -> 0.0
    }

    /**
     * MurmurHash3 (32-bit) — deterministic hash matching the TypeScript implementation.
     */
    fun murmurhash3(key: String, seed: Int = 0): Int {
        var h1 = seed
        val len = key.length
        val c1 = 0xcc9e2d51.toInt()
        val c2 = 0x1b873593

        var i = 0
        while (i + 4 <= len) {
            var k1 = (key[i].code and 0xff) or
                    ((key[i + 1].code and 0xff) shl 8) or
                    ((key[i + 2].code and 0xff) shl 16) or
                    ((key[i + 3].code and 0xff) shl 24)

            k1 *= c1
            k1 = (k1 shl 15) or (k1 ushr 17)
            k1 *= c2

            h1 = h1 xor k1
            h1 = (h1 shl 13) or (h1 ushr 19)
            h1 = h1 * 5 + 0xe6546b64.toInt()

            i += 4
        }

        var k1 = 0
        when (len and 3) {
            3 -> {
                k1 = k1 xor ((key[i + 2].code and 0xff) shl 16)
                k1 = k1 xor ((key[i + 1].code and 0xff) shl 8)
                k1 = k1 xor (key[i].code and 0xff)
                k1 *= c1
                k1 = (k1 shl 15) or (k1 ushr 17)
                k1 *= c2
                h1 = h1 xor k1
            }
            2 -> {
                k1 = k1 xor ((key[i + 1].code and 0xff) shl 8)
                k1 = k1 xor (key[i].code and 0xff)
                k1 *= c1
                k1 = (k1 shl 15) or (k1 ushr 17)
                k1 *= c2
                h1 = h1 xor k1
            }
            1 -> {
                k1 = k1 xor (key[i].code and 0xff)
                k1 *= c1
                k1 = (k1 shl 15) or (k1 ushr 17)
                k1 *= c2
                h1 = h1 xor k1
            }
        }

        h1 = h1 xor len
        h1 = h1 xor (h1 ushr 16)
        h1 *= 0x85ebca6b.toInt()
        h1 = h1 xor (h1 ushr 13)
        h1 *= 0xc2b2ae35.toInt()
        h1 = h1 xor (h1 ushr 16)

        return h1
    }
}

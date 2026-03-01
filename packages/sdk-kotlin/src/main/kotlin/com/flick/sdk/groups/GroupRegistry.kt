package com.flick.sdk.groups

import com.flick.sdk.FlickContext
import java.util.concurrent.ConcurrentHashMap

typealias GroupMatcher = (FlickContext) -> Boolean

class GroupRegistry {
    private val groups = ConcurrentHashMap<String, GroupMatcher>()

    fun register(name: String, matcher: GroupMatcher) {
        groups[name] = matcher
    }

    fun isInGroup(name: String, context: FlickContext): Boolean {
        return groups[name]?.invoke(context) ?: false
    }

    fun getRegisteredGroups(): Set<String> = groups.keys.toSet()
}

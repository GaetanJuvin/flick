package com.flick.sdk.cache

import com.flick.sdk.model.FlagConfig
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

class InMemoryFlagCache {
    private val flags = ConcurrentHashMap<String, FlagConfig>()
    private val lock = ReentrantReadWriteLock()
    @Volatile
    private var version: String? = null

    fun update(newFlags: List<FlagConfig>, newVersion: String): Boolean {
        if (version == newVersion) return false

        lock.write {
            flags.clear()
            newFlags.forEach { flags[it.key] = it }
            version = newVersion
        }
        return true
    }

    fun get(key: String): FlagConfig? = lock.read { flags[key] }

    fun getAll(): List<FlagConfig> = lock.read { flags.values.toList() }

    fun getVersion(): String? = version

    fun isEmpty(): Boolean = flags.isEmpty()
}

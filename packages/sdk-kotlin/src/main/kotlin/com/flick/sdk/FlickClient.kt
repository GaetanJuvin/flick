package com.flick.sdk

import com.flick.sdk.cache.InMemoryFlagCache
import com.flick.sdk.evaluation.Evaluator
import com.flick.sdk.groups.GroupRegistry
import com.flick.sdk.network.HttpClient
import com.flick.sdk.polling.AutoPoller
import kotlinx.coroutines.*
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class FlickClient private constructor(
    private val config: FlickConfig,
) {
    private val cache = InMemoryFlagCache()
    private val groupRegistry = GroupRegistry()
    private val httpClient = HttpClient(config.serverUrl, config.apiKey)
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val readyLatch = CountDownLatch(1)
    private var onFlagsUpdated: (() -> Unit)? = null

    private val poller = AutoPoller(
        httpClient = httpClient,
        intervalSeconds = config.pollingIntervalSeconds,
        onData = { fullConfig ->
            val changed = cache.update(fullConfig.flags, fullConfig.version)
            readyLatch.countDown()
            if (changed) onFlagsUpdated?.invoke()
        },
        onError = { error ->
            System.err.println("Flick polling error: ${error.message}")
            if (cache.isEmpty()) readyLatch.countDown()
        },
    )

    init {
        poller.start(scope)
    }

    fun awaitReady(timeoutSeconds: Long = 10): Boolean {
        return readyLatch.await(timeoutSeconds, TimeUnit.SECONDS)
    }

    fun isEnabled(flagKey: String, context: FlickContext = FlickContext("")): Boolean {
        val flagConfig = cache.get(flagKey)
            ?: return config.defaultValues[flagKey] ?: false

        return Evaluator.evaluate(flagConfig, context).enabled
    }

    fun registerGroup(name: String, matcher: (FlickContext) -> Boolean) {
        groupRegistry.register(name, matcher)
    }

    fun onFlagsUpdated(callback: () -> Unit) {
        this.onFlagsUpdated = callback
    }

    fun shutdown() {
        poller.stop()
        httpClient.close()
        scope.cancel()
    }

    companion object {
        fun create(config: FlickConfig): FlickClient {
            return FlickClient(config)
        }
    }
}

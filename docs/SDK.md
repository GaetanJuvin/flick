# SDK Guide

Flick SDKs are available for TypeScript and Kotlin. Both work the same way: poll the server for flag configs, cache in-memory, evaluate locally.

## How It Works

1. SDK connects to Flick and fetches all flag configs for your environment
2. Configs are cached in memory — `isEnabled()` reads from cache, zero network calls
3. SDK polls every 30s for updates using ETags (304 = no change, zero payload)
4. If Flick goes down, the SDK keeps serving the last known config
5. If Flick is down on cold start, `defaultValues` kick in

Your app never blocks on Flick.

## TypeScript SDK

### Install

```bash
npm install @flick/sdk
```

### Basic Usage

```typescript
import { FlickClient } from '@flick/sdk';

const flick = new FlickClient({
  baseUrl: 'https://flick-server-production.up.railway.app/api/v1',
  sdkKey: 'flk_your_sdk_key_here',
  defaultValues: {
    'new-checkout': false,
    'dark-mode': true,
  },
});

// Wait for initial config fetch (optional but recommended)
await flick.waitForReady();

// Check a flag
if (flick.isEnabled('new-checkout')) {
  renderNewCheckout();
} else {
  renderOldCheckout();
}
```

### With User Context

Pass a context to enable percentage rollouts and group targeting:

```typescript
const enabled = flick.isEnabled('premium-feature', {
  key: user.id,            // used for percentage sticky hashing
  attributes: {
    plan: 'pro',
    country: 'US',
    teams: ['engineering', 'design'],
  },
});
```

### Configuration

```typescript
const flick = new FlickClient({
  // Required
  baseUrl: 'https://your-flick-server.com/api/v1',
  sdkKey: 'flk_...',

  // Optional
  pollingIntervalMs: 30_000,                // default: 30s
  defaultValues: { 'my-flag': false },       // fallback when flag not in cache
  onFlagsUpdated: () => {                    // called when flags change
    console.log('Flags updated');
  },
  onError: (err) => {                        // called on poll failure
    console.error('Flick poll error:', err);
  },
});
```

### Lifecycle

```typescript
// Wait for the SDK to be ready (resolves after first successful poll,
// or immediately if poll fails and cache is empty — returns defaults)
await flick.waitForReady();

// Get all flags as a snapshot
const allFlags = flick.getAllFlags();
// → { 'new-checkout': true, 'dark-mode': false, ... }

// Shut down when your app exits
flick.close();
```

### Resilience

| Scenario | Behavior |
|----------|----------|
| Flick healthy | Flags evaluated from cache, updated every 30s |
| Flick goes down mid-session | Cache keeps serving last known config, exponential backoff on polling (up to 60s) |
| Flick down on cold start | `waitForReady()` resolves immediately, `isEnabled()` returns `defaultValues` |
| No `defaultValues` provided | Unknown flags return `false` (safe — features off by default) |
| Flag not found in cache | Falls back to `defaultValues[key]`, then `false` |

---

## Kotlin SDK

### Gradle

```kotlin
// Add as a local module dependency
implementation(project(":flick-sdk"))
```

### Basic Usage

```kotlin
import com.flick.sdk.FlickClient
import com.flick.sdk.FlickConfig
import com.flick.sdk.FlickContext

val flick = FlickClient.create(
    FlickConfig(
        serverUrl = "https://flick-server-production.up.railway.app/api/v1",
        apiKey = "flk_your_sdk_key_here",
        pollingIntervalSeconds = 30,
        defaultValues = mapOf("new-checkout" to false),
    )
)

// Block until first config is fetched (10s timeout)
flick.awaitReady(timeoutSeconds = 10)

// Check a flag
if (flick.isEnabled("new-checkout")) {
    renderNewCheckout()
}
```

### With Context

```kotlin
val enabled = flick.isEnabled(
    "premium-feature",
    FlickContext(
        key = user.id,
        attributes = mapOf(
            "plan" to "pro",
            "country" to "US",
        ),
    )
)
```

### Group Registration

Register custom group matchers for dynamic targeting:

```kotlin
flick.registerGroup("beta-testers") { context ->
    context.key in betaTesterIds
}
```

### Callbacks

```kotlin
flick.onFlagsUpdated {
    println("Flags changed!")
}
```

### Shutdown

```kotlin
flick.shutdown()  // stops polling, closes HTTP client, cancels coroutines
```

---

## Getting an SDK Key

1. Log in to the Flick dashboard
2. Go to **Settings > API Keys**
3. Create a new key with type **SDK**
4. Bind it to an environment (or pass `X-Environment-Id` header)
5. Copy the key — it's shown only once

SDK keys can only read flag configs via `/evaluate/config`. They cannot create, update, or delete anything.

import { evaluateFlag } from '@flick/shared';
import type { FlagConfig as SharedFlagConfig, EvaluationContext } from '@flick/shared';
import { FlagCache } from './cache.js';
import { Poller } from './polling.js';
import type { FlickClientOptions, FlickContext, FullFlagConfig } from './types.js';

export class FlickClient {
  private cache: FlagCache;
  private poller: Poller;
  private defaultValues: Record<string, boolean>;
  private readyPromise: Promise<void>;
  private resolveReady!: () => void;
  private isReady = false;

  constructor(opts: FlickClientOptions) {
    this.cache = new FlagCache();
    this.defaultValues = opts.defaultValues ?? {};

    this.readyPromise = new Promise((resolve) => {
      this.resolveReady = resolve;
    });

    this.poller = new Poller(
      opts.baseUrl,
      opts.sdkKey,
      opts.pollingIntervalMs ?? 30_000,
      (config: FullFlagConfig) => {
        const changed = this.cache.update(config.flags, config.version);
        if (!this.isReady) {
          this.isReady = true;
          this.resolveReady();
        }
        if (changed && opts.onFlagsUpdated) {
          opts.onFlagsUpdated();
        }
      },
      (error: Error) => {
        if (opts.onError) {
          opts.onError(error);
        }
        // If first poll fails and cache is empty, still resolve ready
        // so callers don't hang forever — they'll get defaults
        if (!this.isReady && this.cache.isEmpty()) {
          this.isReady = true;
          this.resolveReady();
        }
      },
    );

    // Start polling immediately
    this.poller.start();
  }

  async waitForReady(): Promise<void> {
    return this.readyPromise;
  }

  isEnabled(flagKey: string, context?: FlickContext): boolean {
    const flagConfig = this.cache.get(flagKey);

    if (!flagConfig) {
      return this.defaultValues[flagKey] ?? false;
    }

    const evalContext: EvaluationContext = {
      key: context?.key ?? '',
      attributes: (context?.attributes ?? {}) as Record<string, string | number | boolean | string[]>,
    };

    const result = evaluateFlag(flagConfig as unknown as SharedFlagConfig, evalContext);
    return result.enabled;
  }

  getAllFlags(): Record<string, boolean> {
    const result: Record<string, boolean> = { ...this.defaultValues };
    for (const flag of this.cache.getAll()) {
      result[flag.key] = flag.enabled;
    }
    return result;
  }

  close(): void {
    this.poller.stop();
  }
}

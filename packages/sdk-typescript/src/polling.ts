import type { FullFlagConfig } from './types.js';

const MAX_BACKOFF_MS = 60_000;

export class Poller {
  private baseUrl: string;
  private sdkKey: string;
  private intervalMs: number;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private etag: string | null = null;
  private failureCount = 0;
  private stopped = false;

  constructor(
    baseUrl: string,
    sdkKey: string,
    intervalMs: number,
    private onData: (config: FullFlagConfig) => void,
    private onError: (error: Error) => void,
  ) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.sdkKey = sdkKey;
    this.intervalMs = intervalMs;
  }

  async start(): Promise<void> {
    await this.poll();
    this.schedule();
  }

  stop(): void {
    this.stopped = true;
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  private schedule(): void {
    if (this.stopped) return;

    const delay = this.failureCount > 0
      ? Math.min(this.intervalMs * Math.pow(2, this.failureCount), MAX_BACKOFF_MS)
      : this.intervalMs;

    this.timer = setTimeout(async () => {
      await this.poll();
      this.schedule();
    }, delay);
  }

  private async poll(): Promise<void> {
    try {
      const headers: Record<string, string> = {
        'Authorization': `Bearer ${this.sdkKey}`,
        'Accept': 'application/json',
      };
      if (this.etag) {
        headers['If-None-Match'] = this.etag;
      }

      const res = await fetch(`${this.baseUrl}/evaluate/config`, { headers });

      if (res.status === 304) {
        this.failureCount = 0;
        return;
      }

      if (!res.ok) {
        throw new Error(`Polling failed: ${res.status}`);
      }

      const etag = res.headers.get('etag');
      if (etag) this.etag = etag;

      const body = await res.json() as { data: FullFlagConfig };
      this.failureCount = 0;
      this.onData(body.data);
    } catch (err) {
      this.failureCount++;
      this.onError(err instanceof Error ? err : new Error(String(err)));
    }
  }
}

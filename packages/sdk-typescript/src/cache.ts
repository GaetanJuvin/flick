import type { FlagConfig } from './types.js';

export class FlagCache {
  private flags = new Map<string, FlagConfig>();
  private version: string | null = null;

  update(flags: FlagConfig[], version: string): boolean {
    if (this.version === version) return false;

    this.flags.clear();
    for (const flag of flags) {
      this.flags.set(flag.key, flag);
    }
    this.version = version;
    return true;
  }

  get(key: string): FlagConfig | undefined {
    return this.flags.get(key);
  }

  getAll(): FlagConfig[] {
    return Array.from(this.flags.values());
  }

  getVersion(): string | null {
    return this.version;
  }

  isEmpty(): boolean {
    return this.flags.size === 0;
  }
}

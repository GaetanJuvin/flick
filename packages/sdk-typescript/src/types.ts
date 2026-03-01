export interface FlickClientOptions {
  sdkKey: string;
  baseUrl: string;
  pollingIntervalMs?: number;
  defaultValues?: Record<string, boolean>;
  onFlagsUpdated?: () => void;
  onError?: (error: Error) => void;
}

export interface FlickContext {
  key: string;
  attributes?: Record<string, string | number | boolean | string[]>;
}

export interface FlagConfig {
  key: string;
  gate_type: string;
  enabled: boolean;
  gate_config: Record<string, unknown>;
  groups: Array<{
    id: string;
    rules: Array<{
      attribute: string;
      operator: string;
      value: string | string[] | number;
    }>;
  }>;
}

export interface FullFlagConfig {
  environment: string;
  flags: FlagConfig[];
  version: string;
}

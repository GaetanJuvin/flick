export interface EvaluationContext {
  key: string;
  attributes: Record<string, string | number | boolean | string[]>;
}

export type EvaluationReason =
  | 'flag_not_found'
  | 'flag_archived'
  | 'flag_disabled'
  | 'boolean_on'
  | 'percentage_match'
  | 'percentage_miss'
  | 'group_match'
  | 'group_miss'
  | 'default_value';

export interface EvaluationResult {
  flag_key: string;
  enabled: boolean;
  gate_type: string;
  reason: EvaluationReason;
}

export interface BatchEvaluationResult {
  flags: Record<string, EvaluationResult>;
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

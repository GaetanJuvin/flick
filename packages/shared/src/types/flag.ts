export type GateType = 'boolean' | 'percentage' | 'group';

export interface Flag {
  id: string;
  project_id: string;
  key: string;
  name: string;
  description: string;
  gate_type: GateType;
  tags: string[];
  archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateFlagInput {
  key: string;
  name: string;
  description?: string;
  gate_type: GateType;
  tags?: string[];
}

export interface UpdateFlagInput {
  name?: string;
  description?: string;
  tags?: string[];
}

export interface BooleanGateConfig {
  // No extra config for boolean gates
}

export interface PercentageGateConfig {
  percentage: number;
  sticky: boolean;
}

export interface GroupGateConfig {
  // Groups linked via flag_groups table
}

export type GateConfig = BooleanGateConfig | PercentageGateConfig | GroupGateConfig;

export interface FlagEnvironment {
  id: string;
  flag_id: string;
  environment_id: string;
  enabled: boolean;
  gate_config: GateConfig;
  created_at: string;
  updated_at: string;
}

export interface UpdateFlagEnvironmentInput {
  enabled?: boolean;
  gate_config?: GateConfig;
}

export type RuleOperator =
  | 'eq'
  | 'neq'
  | 'in'
  | 'not_in'
  | 'contains'
  | 'starts_with'
  | 'ends_with'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'
  | 'regex';

export interface GroupRule {
  attribute: string;
  operator: RuleOperator;
  value: string | string[] | number;
}

export interface Group {
  id: string;
  project_id: string;
  name: string;
  slug: string;
  description: string;
  rules: GroupRule[];
  created_at: string;
  updated_at: string;
}

export interface CreateGroupInput {
  name: string;
  slug: string;
  description?: string;
  rules: GroupRule[];
}

export interface UpdateGroupInput {
  name?: string;
  description?: string;
  rules?: GroupRule[];
}

export interface FlagGroup {
  id: string;
  flag_environment_id: string;
  group_id: string;
  created_at: string;
}

export interface Environment {
  id: string;
  project_id: string;
  name: string;
  slug: string;
  color: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface CreateEnvironmentInput {
  name: string;
  slug: string;
  color: string;
  sort_order?: number;
}

export interface UpdateEnvironmentInput {
  name?: string;
  color?: string;
  sort_order?: number;
}

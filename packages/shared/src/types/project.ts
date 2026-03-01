export interface Project {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  slug: string;
}

export interface UpdateProjectInput {
  name?: string;
}

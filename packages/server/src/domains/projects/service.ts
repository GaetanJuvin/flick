import type { Project, CreateProjectInput, UpdateProjectInput } from '@flick/shared';
import * as repo from './repo.js';
import * as envRepo from '../environments/repo.js';
import { NotFoundError, ConflictError } from '../../shared/errors.js';

export async function listProjects(): Promise<Project[]> {
  return repo.findAll();
}

export async function getProject(id: string): Promise<Project> {
  const project = await repo.findById(id);
  if (!project) throw new NotFoundError('Project', id);
  return project;
}

export async function createProject(input: CreateProjectInput): Promise<Project> {
  const existing = await repo.findBySlug(input.slug);
  if (existing) throw new ConflictError(`Project with slug '${input.slug}' already exists`);

  const project = await repo.create(input);

  // Create default environments
  const defaults = [
    { name: 'Development', slug: 'development', color: 'blue', sort_order: 0 },
    { name: 'Staging', slug: 'staging', color: 'yellow', sort_order: 1 },
    { name: 'Production', slug: 'production', color: 'red', sort_order: 2 },
  ];

  for (const env of defaults) {
    await envRepo.create(project.id, env);
  }

  return project;
}

export async function updateProject(id: string, input: UpdateProjectInput): Promise<Project> {
  const project = await repo.update(id, input);
  if (!project) throw new NotFoundError('Project', id);
  return project;
}

-- Flick Feature Flag Platform — Initial Schema
-- Gate types: boolean, percentage, group

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- Projects
-- ============================================================
CREATE TABLE projects (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name        VARCHAR(100) NOT NULL,
  slug        VARCHAR(100) NOT NULL UNIQUE,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_slug ON projects(slug);

-- ============================================================
-- Environments (per-project)
-- ============================================================
CREATE TABLE environments (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name        VARCHAR(50) NOT NULL,
  slug        VARCHAR(50) NOT NULL,
  color       VARCHAR(20) NOT NULL DEFAULT 'blue',
  sort_order  INTEGER NOT NULL DEFAULT 0,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(project_id, slug)
);

CREATE INDEX idx_environments_project ON environments(project_id);

-- ============================================================
-- Flags
-- ============================================================
CREATE TABLE flags (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  key         VARCHAR(100) NOT NULL,
  name        VARCHAR(200) NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  gate_type   VARCHAR(20) NOT NULL CHECK (gate_type IN ('boolean', 'percentage', 'group')),
  tags        TEXT[] NOT NULL DEFAULT '{}',
  archived    BOOLEAN NOT NULL DEFAULT false,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(project_id, key)
);

CREATE INDEX idx_flags_project ON flags(project_id);
CREATE INDEX idx_flags_key ON flags(project_id, key);
CREATE INDEX idx_flags_archived ON flags(project_id, archived);

-- ============================================================
-- Flag Environments (core join — per-env flag config)
-- ============================================================
CREATE TABLE flag_environments (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  flag_id         UUID NOT NULL REFERENCES flags(id) ON DELETE CASCADE,
  environment_id  UUID NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  enabled         BOOLEAN NOT NULL DEFAULT false,
  gate_config     JSONB NOT NULL DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(flag_id, environment_id)
);

CREATE INDEX idx_flag_environments_flag ON flag_environments(flag_id);
CREATE INDEX idx_flag_environments_env ON flag_environments(environment_id);

-- ============================================================
-- Groups (named sets of rules)
-- ============================================================
CREATE TABLE groups (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name        VARCHAR(100) NOT NULL,
  slug        VARCHAR(100) NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  rules       JSONB NOT NULL DEFAULT '[]',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(project_id, slug)
);

CREATE INDEX idx_groups_project ON groups(project_id);

-- ============================================================
-- Flag Groups (which groups apply to which flag+env)
-- ============================================================
CREATE TABLE flag_groups (
  id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  flag_environment_id   UUID NOT NULL REFERENCES flag_environments(id) ON DELETE CASCADE,
  group_id              UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(flag_environment_id, group_id)
);

CREATE INDEX idx_flag_groups_flag_env ON flag_groups(flag_environment_id);
CREATE INDEX idx_flag_groups_group ON flag_groups(group_id);

-- ============================================================
-- Users
-- ============================================================
CREATE TABLE users (
  id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  email         VARCHAR(255) NOT NULL UNIQUE,
  name          VARCHAR(100) NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  role          VARCHAR(20) NOT NULL DEFAULT 'viewer' CHECK (role IN ('admin', 'viewer')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users(email);

-- ============================================================
-- User Projects (user-project membership with role)
-- ============================================================
CREATE TABLE user_projects (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  role        VARCHAR(20) NOT NULL DEFAULT 'viewer' CHECK (role IN ('admin', 'viewer')),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(user_id, project_id)
);

CREATE INDEX idx_user_projects_user ON user_projects(user_id);
CREATE INDEX idx_user_projects_project ON user_projects(project_id);

-- ============================================================
-- API Keys (hash-only storage)
-- ============================================================
CREATE TABLE api_keys (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name            VARCHAR(100) NOT NULL,
  key_prefix      VARCHAR(20) NOT NULL,
  key_hash        VARCHAR(64) NOT NULL UNIQUE,
  type            VARCHAR(20) NOT NULL CHECK (type IN ('sdk', 'management')),
  environment_id  UUID REFERENCES environments(id) ON DELETE SET NULL,
  created_by      UUID NOT NULL REFERENCES users(id),
  last_used_at    TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_keys_project ON api_keys(project_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);

-- ============================================================
-- Audit Log (append-only)
-- ============================================================
CREATE TABLE audit_log (
  id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  actor_id      UUID NOT NULL,
  actor_email   VARCHAR(255) NOT NULL,
  action        VARCHAR(50) NOT NULL,
  entity_type   VARCHAR(30) NOT NULL,
  entity_id     VARCHAR(100) NOT NULL,
  entity_name   VARCHAR(200) NOT NULL DEFAULT '',
  before_state  JSONB,
  after_state   JSONB,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_log_project ON audit_log(project_id);
CREATE INDEX idx_audit_log_entity ON audit_log(project_id, entity_type, entity_id);
CREATE INDEX idx_audit_log_actor ON audit_log(project_id, actor_id);
CREATE INDEX idx_audit_log_created ON audit_log(project_id, created_at DESC);

-- ============================================================
-- Webhooks
-- ============================================================
CREATE TABLE webhooks (
  id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  url         VARCHAR(2000) NOT NULL,
  secret      VARCHAR(255) NOT NULL,
  events      TEXT[] NOT NULL DEFAULT '{}',
  status      VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_webhooks_project ON webhooks(project_id);

-- ============================================================
-- Webhook Deliveries (delivery log)
-- ============================================================
CREATE TABLE webhook_deliveries (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  webhook_id       UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
  event            VARCHAR(50) NOT NULL,
  payload          JSONB NOT NULL,
  response_status  INTEGER,
  response_body    TEXT,
  status           VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('success', 'failure', 'pending')),
  attempted_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_webhook_deliveries_webhook ON webhook_deliveries(webhook_id);
CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(webhook_id, status);

-- ============================================================
-- Updated-at trigger function
-- ============================================================
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to all tables with updated_at
CREATE TRIGGER trg_projects_updated_at BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_environments_updated_at BEFORE UPDATE ON environments FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_flags_updated_at BEFORE UPDATE ON flags FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_flag_environments_updated_at BEFORE UPDATE ON flag_environments FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_groups_updated_at BEFORE UPDATE ON groups FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER trg_webhooks_updated_at BEFORE UPDATE ON webhooks FOR EACH ROW EXECUTE FUNCTION update_updated_at();

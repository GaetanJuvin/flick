# Security

## API Keys
- Raw key shown only once at creation
- Stored as SHA-256 hash in database
- Two types: SDK keys (evaluate only) and management keys (full API access)
- Keys are scoped to a project

## Authentication
- Session-based auth for UI (httpOnly cookie)
- API key auth for SDKs and management API
- Passwords hashed with bcrypt (cost factor 12)

## Authorization (RBAC)
- **Admin:** Full access (create/update/delete flags, groups, envs, users)
- **Viewer:** Read-only access (view flags, groups, envs, audit log)
- Per-project roles via `user_projects` join table

## Webhook Security
- HMAC-SHA256 signed payloads
- Signature in `X-Flick-Signature` header
- Webhook secrets stored encrypted

## Data Protection
- Audit log is append-only (no updates or deletes)
- Before/after snapshots for change tracking
- All API inputs validated with Zod schemas

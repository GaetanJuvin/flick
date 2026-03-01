# API Reference

Base URL: `https://flick-server-production.up.railway.app/api/v1`

## Authentication

Two auth methods:

- **Session cookie** — used by the dashboard UI. Set via `POST /auth/login`.
- **API key** — used by SDKs and MCP server. Pass as `Authorization: Bearer flk_...`

## Response Format

All responses follow the same envelope:

```json
// Success (single)
{ "data": { ... } }

// Success (list)
{ "data": [...], "cursor": "...", "has_more": true }

// Error
{ "error": { "code": "NOT_FOUND", "message": "Flag not found" } }
```

---

## Auth

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/auth/config` | None | Auth mode and SAML availability |
| `POST` | `/auth/login` | None | Email/password login, sets session cookie |
| `POST` | `/auth/logout` | None | Clear session cookie |
| `GET` | `/auth/me` | Session | Current user |
| `GET` | `/auth/saml/login` | None | Redirect to SAML IdP |
| `POST` | `/auth/saml/callback` | None | SAML assertion consumer service |

### POST /auth/login

```json
{ "email": "gaetan@juvin.net", "password": "..." }
```

Returns user object and sets `session` cookie (httpOnly, 7 days).

Rate limited: 5 attempts/min per IP, 10 attempts/5min per email.

---

## Profile

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/profile` | Session | Get own profile |
| `PATCH` | `/profile` | Session | Update name/email |
| `POST` | `/profile/password` | Session | Change password |

### POST /profile/password

```json
{ "current_password": "old", "new_password": "new" }
```

Only works for password-auth users (not SAML).

---

## Users (Admin)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/users` | Admin | List all users |
| `POST` | `/users` | Admin | Create user |
| `GET` | `/users/:id` | Session | Get user |
| `PATCH` | `/users/:id` | Admin | Update user |
| `DELETE` | `/users/:id` | Admin | Delete user |
| `POST` | `/users/:id/reset-password` | Admin | Reset user's password |

---

## Projects

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects` | Session | List all projects |
| `POST` | `/projects` | Admin | Create project |
| `GET` | `/projects/:id` | Session | Get project |
| `PATCH` | `/projects/:id` | Admin | Update project |

---

## Environments

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/environments` | Session | List environments |
| `POST` | `/projects/:pid/environments` | Admin | Create environment |
| `PATCH` | `/projects/:pid/environments/:id` | Admin | Update environment |
| `DELETE` | `/projects/:pid/environments/:id` | Admin | Delete environment |

---

## Flags

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/flags` | Session | List flags (query: `archived`, `tags`) |
| `POST` | `/projects/:pid/flags` | Admin | Create flag |
| `GET` | `/projects/:pid/flags/:id` | Session | Get flag |
| `PATCH` | `/projects/:pid/flags/:id` | Admin | Update flag metadata |
| `DELETE` | `/projects/:pid/flags/:id` | Admin | Delete flag |
| `POST` | `/projects/:pid/flags/:id/archive` | Admin | Archive flag |
| `POST` | `/projects/:pid/flags/:id/restore` | Admin | Restore archived flag |

### POST /projects/:pid/flags

```json
{
  "key": "new-checkout",
  "name": "New Checkout",
  "gate_type": "boolean",
  "description": "Redesigned checkout flow"
}
```

Gate types: `boolean`, `percentage`, `group`.

---

## Flag Environments

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/flags/:fid/environments` | Session | List env configs for a flag |
| `GET` | `/projects/:pid/flags/:fid/environments/:eid` | Session | Get one flag-env config |
| `PATCH` | `/projects/:pid/flags/:fid/environments/:eid` | Admin | Update gate config |
| `POST` | `/projects/:pid/flags/:fid/environments/:eid/toggle` | Admin | Toggle flag on/off |

### PATCH gate config example

```json
{
  "gate_type": "percentage",
  "gate_config": { "percentage": 25, "sticky": true }
}
```

---

## Groups

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/groups` | Session | List groups |
| `POST` | `/projects/:pid/groups` | Admin | Create group |
| `GET` | `/projects/:pid/groups/:id` | Session | Get group |
| `PATCH` | `/projects/:pid/groups/:id` | Admin | Update group |
| `DELETE` | `/projects/:pid/groups/:id` | Admin | Delete group |
| `GET` | `/projects/:pid/flags/:fid/environments/:eid/groups` | Session | Groups on a flag-env |
| `POST` | `/projects/:pid/flags/:fid/environments/:eid/groups` | Admin | Attach group |
| `DELETE` | `/projects/:pid/flags/:fid/environments/:eid/groups/:gid` | Admin | Detach group |

### POST group

```json
{
  "name": "Pro Users",
  "slug": "pro-users",
  "rules": [
    { "attribute": "plan", "operator": "eq", "value": "pro" },
    { "attribute": "country", "operator": "in", "value": ["US", "CA"] }
  ]
}
```

Rules within a group are AND'd. Multiple groups on a flag are OR'd.

---

## Evaluation (SDK)

These endpoints use API key auth and are rate-limited (1000 req/10s per key).

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/evaluate` | API Key | Evaluate one flag |
| `POST` | `/evaluate/batch` | API Key | Evaluate all flags |
| `GET` | `/evaluate/config` | API Key | Full flag config (for SDK polling) |

### POST /evaluate

```json
{
  "flag_key": "new-checkout",
  "context": {
    "key": "user-123",
    "attributes": { "plan": "pro" }
  }
}
```

### GET /evaluate/config

Returns the full config payload for SDK polling. Supports `ETag` / `If-None-Match` for efficient 304 responses.

```json
{
  "data": {
    "environment": "production",
    "version": "abc123",
    "flags": [
      {
        "key": "new-checkout",
        "gate_type": "boolean",
        "enabled": true,
        "gate_config": {},
        "groups": []
      }
    ]
  }
}
```

---

## Audit Log

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/audit` | Session | Query audit log |

Query params: `entity_type`, `entity_id`, `actor_id`, `action`, `cursor`, `limit`.

---

## API Keys

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/api-keys` | Admin | List keys (secrets masked) |
| `POST` | `/projects/:pid/api-keys` | Admin | Create key (plaintext shown once) |
| `DELETE` | `/projects/:pid/api-keys/:id` | Admin | Revoke key |

Key types: `sdk` (read-only evaluate) or `management` (full access).

---

## Webhooks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/projects/:pid/webhooks` | Session | List webhooks |
| `POST` | `/projects/:pid/webhooks` | Admin | Create webhook |
| `PATCH` | `/projects/:pid/webhooks/:id` | Admin | Update webhook |
| `DELETE` | `/projects/:pid/webhooks/:id` | Admin | Delete webhook |
| `POST` | `/projects/:pid/webhooks/:id/test` | Admin | Send test delivery |
| `GET` | `/projects/:pid/webhooks/:id/deliveries` | Session | Delivery history |

Webhook payloads are signed with HMAC-SHA256 via the `X-Flick-Signature` header.

---

## Rate Limits

| Endpoint | Limit | Window | Key |
|----------|-------|--------|-----|
| `POST /auth/login` | 5 requests | 60s | Per IP |
| `POST /auth/login` | 10 requests | 5min | Per email |
| `POST /evaluate` | 1000 requests | 10s | Per API key |
| `POST /evaluate/batch` | 1000 requests | 10s | Per API key |
| `GET /evaluate/config` | 1000 requests | 10s | Per API key |

Rate limit headers on every response:
- `X-RateLimit-Limit` — max requests in window
- `X-RateLimit-Remaining` — remaining requests
- `X-RateLimit-Reset` — Unix timestamp when window resets
- `Retry-After` — seconds until retry (only on 429)

---

## Health Check

```
GET /health → { "status": "ok" }
```

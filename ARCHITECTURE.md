# Architecture

## System Overview

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Astro UI    │────▶│  Fastify API │◀────│  MCP Server  │
│  (Dashboard) │     │  (Server)    │     │  (AI Agents) │
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                     ┌──────┴───────┐
                     │              │
                ┌────▼────┐  ┌─────▼─────┐
                │ Postgres│  │   Redis   │
                │ (Data)  │  │  (Cache)  │
                └─────────┘  └───────────┘
                            ▲
              ┌─────────────┼─────────────┐
              │             │             │
        ┌─────▼─────┐ ┌────▼──────┐ ┌────▼──────┐
        │ TS SDK    │ │ Kotlin SDK│ │ Other SDKs│
        │ (polling) │ │ (polling) │ │  (future) │
        └───────────┘ └───────────┘ └───────────┘
```

## Tech Stack

- **Server:** Node.js + Fastify
- **UI:** Astro SSR + React islands + Tailwind CSS
- **Database:** PostgreSQL (data) + Redis (cache)
- **Build:** pnpm + Turborepo
- **SDKs:** TypeScript (polling), Kotlin (coroutines)

## Gate Types (v1)

| Gate | Evaluation | Config |
|------|-----------|--------|
| **Boolean** | Simple on/off | `{}` |
| **Percentage** | `murmurhash3(flagKey + contextKey) % 100 < percentage` | `{ percentage: number, sticky: boolean }` |
| **Group** | Context attributes matched against group rules | `{}` (groups linked via `flag_groups`) |

## System Layers

```
┌─────────────────────────────────────────┐
│            UI Layer (Astro)              │
│  Pages, React islands, components        │
├─────────────────────────────────────────┤
│          Routes Layer (Fastify)          │
│  HTTP handlers, request/response         │
├─────────────────────────────────────────┤
│         Service Layer                    │
│  Business logic, orchestration           │
├─────────────────────────────────────────┤
│        Repository Layer                  │
│  Data access (PostgreSQL queries)        │
├─────────────────────────────────────────┤
│          Cache Layer (Redis)             │
│  Write-through cache, TTL, invalidation  │
├─────────────────────────────────────────┤
│          Types Layer (@flick/shared)     │
│  Shared types, Zod schemas               │
└─────────────────────────────────────────┘
```

**Dependency rule:** Dependencies flow downward only.

## Domain Module Pattern

Each server domain in `packages/server/src/domains/` follows:

```
domains/[name]/
├── types.ts     # Domain-specific types and enums
├── repo.ts      # PostgreSQL queries (data access)
├── service.ts   # Business logic (calls repo, triggers events)
└── routes.ts    # Fastify route handlers (calls service)
```

## Core Data Model

- `flag_environments` is the **core join** — flags have no state without an environment
- `gate_config` is JSONB for extensibility
- Group rules are JSONB arrays — rules within a group are ANDed, groups on a flag are ORed
- API keys store SHA-256 hash only; raw key shown once at creation
- Audit log stores before/after JSONB snapshots

## API Response Envelope

```typescript
// Success
{ data: T }

// List
{ data: T[], cursor: string | null, has_more: boolean }

// Error
{ error: { code: string, message: string } }
```

## Caching Strategy

| Cache Key | TTL | Invalidation |
|-----------|-----|-------------|
| `flick:env:{envId}:flags` | 60s | On any flag/group change in that env |
| `flick:env:{envId}:flag:{key}` | 60s | On specific flag change |
| `flick:apikey:{hash}` | 300s | On key revocation |

Write-through: miss → fetch from Postgres → write to Redis → return.

## SDK Architecture

Both SDKs follow the same pattern:
1. Poll `GET /evaluate/config` with ETag for 304 support
2. Cache full flag config in memory
3. Evaluate locally (zero-latency) using shared evaluation logic
4. Exponential backoff on failures, serve from cache during outages

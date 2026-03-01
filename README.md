# Flick

Internal feature flag platform. Replace LaunchDarkly with something you own.

**Dashboard:** https://flick-ui-production.up.railway.app
**API:** https://flick-server-production.up.railway.app

## What is Flick?

Flick lets you control feature rollouts across environments without deploying code. Create a flag, target users with rules, and toggle features on or off instantly.

- **Boolean flags** — simple on/off
- **Percentage rollouts** — gradual rollout to N% of users (sticky via murmurhash)
- **Group targeting** — define rules like `plan = pro AND country IN [US, CA]`
- **Per-environment config** — different flag states for dev, staging, production
- **Audit log** — every change is recorded with before/after snapshots

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Astro UI   │────▶│  Fastify API │◀────│  MCP Server  │
│  (React)    │     │              │     │  (AI agents) │
└─────────────┘     └──────┬───────┘     └─────────────┘
                           │
                    ┌──────┴───────┐
                    │              │
               ┌────▼────┐  ┌─────▼────┐
               │Postgres │  │  Redis   │
               └─────────┘  └──────────┘
                    ▲
          ┌────────┴─────────┐
          │                  │
    ┌─────▼──────┐   ┌──────▼─────┐
    │  TS SDK    │   │ Kotlin SDK │
    │ (polling)  │   │ (polling)  │
    └────────────┘   └────────────┘
```

SDKs poll the API for flag configs, cache them in-memory, and evaluate locally. Your app never blocks on Flick — if the server goes down, SDKs keep serving the last known config.

## Quick Start

```bash
git clone git@github.com:GaetanJuvin/flick.git
cd flick
pnpm install
docker compose up -d          # PostgreSQL + Redis
cp .env.example .env          # edit as needed
pnpm --filter @flick/server db:migrate
pnpm --filter @flick/server dev
pnpm --filter @flick/ui dev
```

Open http://localhost:4321 and log in.

## Packages

| Package | Path | Description |
|---------|------|-------------|
| `@flick/server` | `packages/server` | Fastify API server |
| `@flick/ui` | `packages/ui` | Astro dashboard with React islands |
| `@flick/shared` | `packages/shared` | Types + Zod schemas shared across packages |
| `@flick/sdk` | `packages/sdk-typescript` | TypeScript SDK (polling + local eval) |
| `@flick/mcp-server` | `packages/mcp-server` | MCP server for AI agents |
| Kotlin SDK | `packages/sdk-kotlin` | Kotlin SDK (coroutine-based polling) |

## Documentation

| Doc | Description |
|-----|-------------|
| [SDK Guide](docs/SDK.md) | TypeScript & Kotlin SDK integration |
| [MCP Server](docs/MCP.md) | AI agent integration via Model Context Protocol |
| [API Reference](docs/API.md) | REST API endpoints |
| [Architecture](ARCHITECTURE.md) | System design, layers, data model |
| [Security](docs/SECURITY.md) | Auth, RBAC, API keys, audit |
| [Design Patterns](docs/DESIGN.md) | Code conventions and patterns |
| [Frontend](docs/FRONTEND.md) | Astro + React island architecture |

## Commands

```bash
pnpm dev          # Run all packages in dev mode
pnpm build        # Build all packages
pnpm lint         # Lint all packages
pnpm typecheck    # Type-check all packages
pnpm test         # Run all tests
```

## License

Private. Internal use only.

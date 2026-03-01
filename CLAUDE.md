# Flick — Feature Flag Platform

> Internal feature flag platform replacing LaunchDarkly. Inspired by Flipper's simplicity.

## Quick Start

```bash
pnpm install
docker compose up -d          # PostgreSQL + Redis
pnpm --filter @flick/server dev
pnpm --filter @flick/ui dev
```

## Repository Map

| Path | Package | Purpose |
|------|---------|---------|
| `packages/shared` | `@flick/shared` | Types + Zod schemas shared across all packages |
| `packages/server` | `@flick/server` | Fastify API server |
| `packages/ui` | `@flick/ui` | Astro dashboard with React islands |
| `packages/sdk-typescript` | `@flick/sdk` | TypeScript SDK (polling-based) |
| `packages/sdk-kotlin` | — | Kotlin SDK (coroutine-based polling) |
| `packages/mcp-server` | `@flick/mcp-server` | MCP server for AI agents |

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for system layers, dependency rules, and gate types.

## Key Concepts

- **Gate Types:** Boolean, Percentage of Time, Groups
- **Environments:** Flags have per-environment configuration (dev, staging, production)
- **Groups:** Named sets of rules (AND within group, OR across groups on a flag)
- **Evaluation:** `flag_environments` is the core join — flags have no state without an environment

## Domain Module Pattern

Each server domain follows: `types.ts → repo.ts → service.ts → routes.ts`

Dependencies flow downward only: Types → Config → Repo → Service → Routes

## Docs

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System layers and dependency rules |
| [docs/DESIGN.md](./docs/DESIGN.md) | Design patterns and conventions |
| [docs/FRONTEND.md](./docs/FRONTEND.md) | Frontend architecture (Astro + React) |
| [docs/SECURITY.md](./docs/SECURITY.md) | Security policies |
| [docs/RELIABILITY.md](./docs/RELIABILITY.md) | Reliability standards |
| [docs/QUALITY_SCORE.md](./docs/QUALITY_SCORE.md) | Quality grades per domain |

## Commands

- `pnpm dev` — Run all packages in dev mode
- `pnpm build` — Build all packages
- `pnpm lint` — Lint all packages
- `pnpm typecheck` — Type-check all packages
- `pnpm test` — Run all tests
- `scripts/lint-architecture.sh` — Enforce architectural boundaries

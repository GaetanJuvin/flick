# Design Patterns & Conventions

## Domain Module Pattern

Every server domain follows a strict 4-file pattern:

```
domains/[name]/
├── types.ts     # Domain types, enums, Zod schemas (import from @flick/shared)
├── repo.ts      # Data access — raw SQL via pg, returns typed objects
├── service.ts   # Business logic — calls repo, triggers audit/cache/webhooks
└── routes.ts    # HTTP handlers — validation, auth, calls service
```

## API Conventions

- All routes prefixed with `/api/v1`
- Response envelope: `{ data }` / `{ data[], cursor, has_more }` / `{ error: { code, message } }`
- Cursor-based pagination (opaque base64 cursor)
- Zod validation on all request bodies

## Naming

- Database: `snake_case` for tables and columns
- TypeScript: `camelCase` for variables, `PascalCase` for types
- API: `snake_case` in JSON payloads (matching DB)
- URL paths: `kebab-case`
- Flag keys: `kebab-case` (enforced by schema)

## Error Handling

- Domain errors extend `AppError` with a code and HTTP status
- Routes catch errors and return envelope format
- Never expose internal details in error messages

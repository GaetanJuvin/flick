# Core Beliefs

1. **Repository Is the Source of Truth** — All decisions committed to code
2. **Validate at Boundaries, Trust Internally** — Zod schemas at API boundaries
3. **Prefer Shared Utilities** — Check `@flick/shared` before hand-rolling
4. **Make It Legible** — Code should teach the next reader
5. **Architecture is Non-Negotiable** — Layer rules enforced by linter
6. **Small, Focused Changes** — Many small PRs over few large ones
7. **Boring Technology Wins** — Prefer stable, documented libraries
8. **Flag Evaluation Must Be Fast** — Local evaluation in SDKs, zero-latency
9. **Audit Everything** — Every state change logged with before/after snapshots
10. **Graceful Degradation** — SDKs serve stale data over failing hard

# Frontend Architecture

## Stack

- **Astro** SSR mode (`output: 'server'`) with Node adapter
- **React** islands for interactive components
- **Tailwind CSS** for styling

## Component Strategy

### Static Astro (zero JS)
Layout, navigation, cards, badges, and read-only displays are Astro components. No JavaScript shipped to the client.

### React Islands
Interactive components use React with specific hydration directives:
- `client:load` — Needed immediately (filters, configurators, tabs)
- `client:visible` — Loaded when scrolled into view (toggles, tables)
- `client:idle` — Loaded after page is idle (modals, forms)

## Data Flow

1. Astro pages fetch data server-side from the Fastify API
2. React islands receive data as props
3. Mutations use `fetch()` to the API, then invalidate/refetch
4. Optimistic updates for toggles and simple actions

## Auth

Astro middleware checks session cookie on all routes. Populates `Astro.locals.user`.

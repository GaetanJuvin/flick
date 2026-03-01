# MCP Server

Flick ships an MCP (Model Context Protocol) server that lets AI agents — Claude, Cursor, Windsurf, etc. — manage feature flags through structured tools.

## What Can AI Agents Do?

With the MCP server, an AI agent can:

- List, create, toggle, and archive feature flags
- Manage environments and targeting groups
- Evaluate flags for specific user contexts
- Read the audit log
- Browse projects, flags, and configs as resources

## Setup

### Prerequisites

- Node.js 22+
- A Flick **Management** API key (not an SDK key)

### Build

```bash
cd packages/mcp-server
pnpm install
pnpm build
```

### Configure in Claude Code

Add to your `~/.claude/claude_code_config.json`:

```json
{
  "mcpServers": {
    "flick": {
      "command": "node",
      "args": ["/path/to/flick/packages/mcp-server/dist/index.js"],
      "env": {
        "FLICK_API_URL": "https://flick-server-production.up.railway.app/api/v1",
        "FLICK_API_KEY": "flk_your_management_key_here"
      }
    }
  }
}
```

### Configure in Cursor / Windsurf

Add to your project's `.cursor/mcp.json` or equivalent:

```json
{
  "mcpServers": {
    "flick": {
      "command": "node",
      "args": ["./packages/mcp-server/dist/index.js"],
      "env": {
        "FLICK_API_URL": "https://flick-server-production.up.railway.app/api/v1",
        "FLICK_API_KEY": "flk_your_management_key_here"
      }
    }
  }
}
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FLICK_API_URL` | `http://localhost:3000/api/v1` | Flick API base URL |
| `FLICK_API_KEY` | — | Management API key (required) |

---

## Tools

### Flags (7 tools)

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `flick_list_flags` | List all flags in a project | `project_id` |
| `flick_get_flag` | Get flag details | `project_id`, `flag_id` |
| `flick_create_flag` | Create a new flag | `project_id`, `key`, `name`, `gate_type` |
| `flick_toggle_flag` | Toggle flag on/off for an environment | `project_id`, `flag_id`, `env_id` |
| `flick_update_flag_config` | Update gate config (percentage, etc.) | `project_id`, `flag_id`, `env_id`, `gate_config` |
| `flick_archive_flag` | Archive a flag | `project_id`, `flag_id` |
| `flick_evaluate_flag` | Evaluate a flag for a context | `env_id`, `flag_key`, `context_key`, `attributes` |

### Environments (1 tool)

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `flick_list_environments` | List all environments | `project_id` |

### Groups (4 tools)

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `flick_list_groups` | List all groups | `project_id` |
| `flick_create_group` | Create a group with rules | `project_id`, `name`, `slug`, `rules` |
| `flick_add_group_to_flag` | Attach a group to a flag/env | `project_id`, `flag_id`, `env_id`, `group_id` |
| `flick_remove_group_from_flag` | Detach a group from a flag/env | `project_id`, `flag_id`, `env_id`, `group_id` |

### Audit (1 tool)

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `flick_get_audit_log` | View audit log entries | `project_id`, `entity_type`, `action`, `limit` |

---

## Resources

Resources are read-only data surfaces. AI agents can inspect these without making mutations.

| Resource | URI | Description |
|----------|-----|-------------|
| Projects | `flick://projects` | All projects |
| Flags | `flick://projects/{slug}/flags` | All flags in a project |
| Environments | `flick://projects/{slug}/environments` | All environments |
| Groups | `flick://projects/{slug}/groups` | All groups |
| Audit Log | `flick://projects/{slug}/audit` | Last 20 audit entries |

---

## Example Conversations

### "Create a feature flag for dark mode"

The agent will:
1. Call `flick_list_flags` to check if it already exists
2. Call `flick_create_flag` with `key: "dark-mode"`, `gate_type: "boolean"`
3. Call `flick_list_environments` to find the dev environment
4. Call `flick_toggle_flag` to enable it in dev

### "Roll out the new checkout to 25% of users in production"

The agent will:
1. Call `flick_list_flags` to find the checkout flag
2. Call `flick_list_environments` to find production
3. Call `flick_update_flag_config` with `gate_config: { percentage: 25, sticky: true }`
4. Call `flick_toggle_flag` to enable it

### "Who changed the signup-flow flag?"

The agent will:
1. Call `flick_get_audit_log` with `entity_type: "flag"` and the project ID
2. Filter results for the `signup-flow` flag and report the changes

---

## API Key Types

| Type | Can Read | Can Mutate | Use Case |
|------|----------|------------|----------|
| **SDK** | `/evaluate/*` only | No | SDKs polling for flag configs |
| **Management** | Everything | Yes | Dashboard, MCP server, CI/CD |

The MCP server requires a **Management** key. SDK keys will get 403 errors on mutation endpoints.

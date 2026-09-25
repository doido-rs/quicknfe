# QuickNFe documentation

QuickNFe is a multi-tenant fiscal document management app: **Rust/Doido backend**, **React/Vite frontend**, optional **MCP + OAuth** for AI clients.

## Documentation map

| Section | Description |
|---------|-------------|
| [Harness (dev workflow)](harness/README.md) | Makefile, Docker, envs, CI — how to run and test the project |
| [Architecture](architecture/README.md) | System overview, tenancy, stack |
| [Getting started](guides/getting-started.md) | First-time setup from zero |
| [MCP integration](guides/mcp.md) | HTTP MCP server and OAuth |
| [API reference (Swagger)](api/README.md) | OpenAPI + interactive Swagger UI |

## Quick links

- **Run locally:** `make setup && make dev` → API `:3000`, UI `:5173`
- **Swagger:** [http://localhost:3000/api-docs](http://localhost:3000/api-docs)
- **Seed user:** `admin@quicknfe.local` / `QuickNFe2026!`
- **Backend details:** [../backend/README.md](../backend/README.md)
- **Frontend details:** [../frontend/README.md](../frontend/README.md)

## Repository layout

```
quicknfe/
├── backend/          # Doido app (Axum, SeaORM, routes, MCP, OAuth)
├── frontend/         # React SPA (Vite, shadcn/ui)
├── docs/             # This documentation tree
├── docs/api/         # OpenAPI spec + Swagger assets
├── docker-compose.yml
└── Makefile          # Primary dev harness entrypoint
```

# QuickNFe API documentation

## Swagger UI (interactive)

With the backend running:

| URL | Description |
|-----|-------------|
| [http://localhost:3000/api-docs](http://localhost:3000/api-docs) | Swagger UI |
| [http://localhost:3000/api-docs/openapi.yaml](http://localhost:3000/api-docs/openapi.yaml) | OpenAPI 3.0 spec |

### Try it out

1. Start the stack: `make dev` or `make dev-backend`.
2. Open **http://localhost:3000/api-docs** (same origin so cookies work).
3. Call **Auth → POST /users/sign_in** with seed credentials (`admin@quicknfe.local` / `QuickNFe2026!`).
4. Call **Tenant → POST /tenant/empresa** with `{ "empresa_id": <id> }` from `/api/me`.
5. Use **Session** and resource endpoints under `/api/*`.

The spec source of truth is [`openapi.yaml`](openapi.yaml). The server embeds this file at build time.

## Offline / static

Open [`swagger.html`](swagger.html) in a browser only works if you serve the folder (e.g. `npx serve docs/api`) and point the UI at `openapi.yaml`.

## MCP & OAuth

REST `/api/*` uses **session cookies**. **MCP** (`POST /mcp`) uses **Bearer OAuth** — documented in the same OpenAPI under tags MCP and OAuth. See [../guides/mcp.md](../guides/mcp.md).

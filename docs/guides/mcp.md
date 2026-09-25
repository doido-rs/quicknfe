# MCP (Model Context Protocol)

QuickNFe exposes an optional **Streamable HTTP MCP** server on the same port as the REST API.

## Enable

Development defaults (see `backend/.env.example`):

```env
MCP_ENABLED=true
MCP_PUBLIC_URL=http://localhost:3000
MCP_OAUTH_SIGNING_SECRET=<long random string>
```

Run migrations so `oauth_*` tables exist.

## Endpoints

| Path | Purpose |
|------|---------|
| `POST /mcp` | MCP protocol (requires Bearer token) |
| `GET /.well-known/oauth-protected-resource` | RFC 9728 PRM |
| `GET /.well-known/oauth-authorization-server` | RFC 8414 metadata |
| `GET /oauth/authorize` | Browser login + authorize |
| `POST /oauth/token` | PKCE token exchange |
| `POST /oauth/register` | Register OAuth client (dev) |

Dev client id (seeded): `quicknfe_mcp_dev`

## Cursor / remote clients

1. Point MCP URL to `http://localhost:3000/mcp`
2. Complete OAuth when prompted (browser opens authorize page)
3. Use tools `quicknfe_switch_empresa` then resource tools (`quicknfe_clientes_list`, etc.)

Tools mirror REST routes; generic `quicknfe_request` accepts allowlisted method + path.

## Security notes

- REST `/api/*` remains **cookie session** for the SPA.
- MCP uses **OAuth JWT**; session cookie is stored server-side per token.
- Production requires HTTPS `MCP_PUBLIC_URL`.

Implementation: `backend/app/mcp/`, `backend/app/oauth/`.

Tests: `DOIDO_ENV=test cargo test --test mcp_oauth_test -- --test-threads=1`

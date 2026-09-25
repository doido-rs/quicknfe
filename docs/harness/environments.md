# Environments and configuration

## DOIDO_ENV

Doido loads `backend/config/<DOIDO_ENV>.yml`. Default when unset: **development**.

| Environment | Typical use |
|-------------|-------------|
| `development` | Local `make dev`, `.env` loaded via `startup::prepare` |
| `test` | `cargo test`, integration tests |
| `production` | Kamal / Docker image |

## Key settings

### Database

`database.url` in YAML uses Tera `get_env` for `DATABASE_URL`.

### Auth

From `development.yml` / `test.yml`:

- User model: `User`
- Cookie session strategy
- Routes under `/users` (`sign_in`, `sign_up`, `sign_out`, …)

### CORS

Development allows `http://localhost:5173` with credentials for the SPA.

### MCP (development)

When `MCP_ENABLED` is true (default in development):

- `MCP_PUBLIC_URL` — public base URL for OAuth metadata
- `MCP_OAUTH_SIGNING_SECRET` — JWT signing (required for production)

See [../guides/mcp.md](../guides/mcp.md).

## Frontend

- `VITE_API_URL` — empty uses Vite proxy (recommended in dev)
- Theme preference: `localStorage` key `quicknfe-theme` (`light` | `dark`)

## Secrets

- Do not commit `backend/.env` or `config/master.key`
- Use `.env.example` as template

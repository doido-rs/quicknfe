# CI and deployment

## Local CI

```bash
make ci
```

Runs:

1. `make lint` — oxlint, `cargo fmt --check`, clippy
2. `make test-all` — unit + `tenant_api_test` (+ other integration tests in backend crate)
3. `make build` — release binary + frontend production bundle

Ensure MySQL is running and test DB migrated before relying on `test-all` in a clean machine:

```bash
make up
cd backend && DOIDO_ENV=test cargo doido db create && cargo doido db migrate
```

Additional MCP tests:

```bash
DOIDO_ENV=test cargo test --test mcp_oauth_test -- --test-threads=1
```

## Docker image

```bash
make docker-build   # docker build -t quicknfe backend/
```

Uses `backend/Dockerfile` (distroless runtime in production path).

## Kamal deploy

Config: `backend/config/deploy.yml`

```bash
make deploy   # kamal deploy -c backend/config/deploy.yml
```

Edit registry, hosts, and env in `deploy.yml` before first deploy. Render production secrets via Doido credentials or orchestrator env.

## Documentation in deploy

- Serve OpenAPI at `/api-docs` from the same backend process (no extra service).
- For HTTPS production, set `MCP_PUBLIC_URL` to your public origin if MCP is enabled.

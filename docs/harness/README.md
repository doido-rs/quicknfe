# Development harness

The **harness** is everything you use to build, run, test, and deploy QuickNFe locally and in CI. The main entrypoint is the root [Makefile](../../Makefile).

## Prerequisites

| Tool | Purpose |
|------|---------|
| [Rust](https://rustup.rs) + `mise` (see `backend/mise.toml`) | Backend compile |
| [doido CLI](https://github.com/) | `cargo doido server`, migrations, seeds |
| cargo-watch | `make dev-backend` reloads on Rust/config changes |
| [Bun](https://bun.sh) | Frontend install & tests |
| Docker + Compose | MySQL, Mailpit |

## Makefile targets

Run `make help` for the full list. Common flows:

| Target | What it does |
|--------|----------------|
| `make setup` | Docker MySQL + Mailpit, `bun install`, `db create`, `migrate`, `seed` |
| `make dev` | `dev-backend` + `dev-frontend` in parallel |
| `make dev-backend` | API on **http://localhost:3000** (cargo-watch) |
| `make dev-frontend` | Vite on **http://localhost:3001** (proxies `/api`, `/users`, `/tenant`) |
| `make lint` | oxlint + rustfmt/clippy |
| `make fix` | Auto-fix lint where possible |
| `make test` | Unit tests (frontend + backend crate) |
| `make test-integration` | `tenant_api_test` against MySQL `quicknfe_test` |
| `make test-all` | Unit + integration |
| `make ci` | lint + test-all + production build |
| `make deploy` | Kamal using `backend/config/deploy.yml` |

See [makefile.md](makefile.md) for notes on each target.

## Docker Compose

[`docker-compose.yml`](../../docker-compose.yml) provides:

- **mysql** — port `3306`, database `quicknfe_development`, root password `password`
- **mailpit** — SMTP `1025`, UI **http://localhost:8025**

The `web` service is commented out; typical dev runs the backend on the host with `make dev-backend`.

Details: [docker-compose.md](docker-compose.md).

## Environments

| `DOIDO_ENV` | Config file | Database (default) |
|-------------|-------------|-------------------|
| `development` | `backend/config/development.yml` | `quicknfe_development` |
| `test` | `backend/config/test.yml` | `quicknfe_test` |
| `production` | `backend/config/production.yml` | From env / deploy |

Copy `backend/.env.example` → `backend/.env` for local variables.

Details: [environments.md](environments.md).

## CI pipeline (local)

`make ci` mirrors a minimal pipeline:

1. Lint frontend and backend
2. Run all tests (including MySQL integration)
3. Release build (Rust + frontend static)

Details: [ci-cd.md](ci-cd.md).

## API docs in the harness

When the backend is up, open **Swagger UI** at `/api-docs`. The OpenAPI file lives in [../api/openapi.yaml](../api/openapi.yaml).

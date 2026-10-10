# Makefile reference

Root Makefile variables:

- `BACKEND_DIR=backend`
- `FRONTEND_DIR=frontend`
- `COMPOSE=docker compose`

## Environment setup

```bash
make setup    # Recommended first run: containers + DB + seed
make install  # Frontend deps only
make up       # MySQL + Mailpit
make down     # Stop compose
```

Database (development):

```bash
make db-create
make db-migrate
make db-seed
make db-setup   # create + migrate
make db-console # MySQL CLI (quicknfe_development)
```

## Development servers

```bash
make dev           # Both backend and frontend (hot reload)
make dev-backend   # cargo watch → cargo doido server → :3000
make dev-frontend  # bun run dev (Vite HMR) → :3001
```

Frontend proxy (see `frontend/vite.config.ts`): `/api`, `/users`, `/tenant` → backend. Add `/api-docs` if you proxy Swagger through Vite.

## Quality

```bash
make lint
make fix
make test
make test-integration   # Requires test DB migrated
make test-all
make test-coverage      # Frontend 80% thresholds
```

Integration test prep:

```bash
DOIDO_ENV=test cargo doido db create   # from backend/
DOIDO_ENV=test cargo doido db migrate
make test-integration
```

## Build & deploy

```bash
make build        # cargo release + bun build
make docker-build
make deploy       # Kamal
```

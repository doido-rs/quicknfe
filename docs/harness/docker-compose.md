# Docker Compose stack

File: [`docker-compose.yml`](../../docker-compose.yml) (project name `quicknfe`).

## Services

### mysql

- Image: `mysql:lts`
- Port: **3306**
- Credentials: user `root`, password `password`
- Default database: `quicknfe_development`
- Healthcheck enabled for dependent services

Connection string (host dev):

```
mysql://root:password@localhost:3306/quicknfe_development
```

### mailpit

- SMTP: **localhost:1025** (backend `development.yml` uses this)
- Web UI: **http://localhost:8025**

### web (optional, commented)

Production-style container build is disabled in compose; use `make dev-backend` on the host instead.

## Commands

```bash
docker compose up mysql mailpit -d   # same as make up
docker compose down
docker compose logs -f mysql
```

## Volumes

Named volumes for Cargo cache exist for the commented `web` service; host dev uses local `target/`.

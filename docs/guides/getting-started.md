# Getting started

## 1. Clone and install tools

- Rust (see `backend/mise.toml`)
- [Bun](https://bun.sh)
- Docker
- Doido CLI (for `cargo doido` in backend)

## 2. One-command setup

From repository root:

```bash
make setup
```

This starts MySQL and Mailpit, installs frontend deps, creates/migrates/seeds the development database.

## 3. Run

```bash
make dev
```

| Service | URL |
|---------|-----|
| Frontend | http://localhost:5173 |
| Backend API | http://localhost:3000 |
| Swagger UI | http://localhost:3000/api-docs |
| Mailpit | http://localhost:8025 |

## 4. Log in

Seed credentials:

- Email: `admin@quicknfe.local`
- Password: `QuickNFe2026!`

Flow: **Login** → **Select empresa** → **App** sidebar resources.

## 5. Explore the API

Open [http://localhost:3000/api-docs](http://localhost:3000/api-docs):

1. Execute `POST /users/sign_in`
2. Execute `POST /tenant/empresa` with your empresa id
3. Try `GET /api/clientes`

## 6. Run tests

```bash
make test-all
```

## Next steps

- [Harness documentation](../harness/README.md)
- [Architecture](../architecture/README.md)
- [MCP for AI clients](mcp.md)

# Technology stack

## Backend

- **Language:** Rust 2021
- **Framework:** [Doido](https://github.com/) (Axum, SeaORM, auth, jobs, mailer, storage)
- **Database:** MySQL (SeaORM migrations in `backend/db/migration/`)
- **MCP:** `rmcp` Streamable HTTP + embedded OAuth 2.1
- **API docs:** OpenAPI YAML + Swagger UI (`backend/app/api_docs/`)

## Frontend

- **Runtime:** Bun
- **UI:** React, React Router, TanStack Query
- **Build:** Vite 6
- **Styling:** Tailwind CSS v4, shadcn/ui, Geist font
- **Tests:** Vitest, Testing Library, jsdom

## Infrastructure

- **Local DB/mail:** Docker Compose (MySQL, Mailpit)
- **Deploy:** Kamal (`backend/config/deploy.yml`)
- **CI harness:** Root Makefile (`make ci`)

## External integrations

- **NFSe transmission** — outbound HTTP from `services/nfse` when transmitting documentos
- **SMTP** — Mailpit in dev; configurable in YAML for production

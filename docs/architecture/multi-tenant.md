# Multi-tenant model

## Concepts

- **User** — authenticates via Doido (`users` table).
- **Company** — tenant organization (`companies` table); session stores active company id.
- **Membership** — `companies_usuarios` links users to companies with a `papel` (role).
- **Empresa (prestador)** — fiscal service provider (`empresas` table), belongs to a company via `company_id`. Many prestadores per company. Session stores active prestador id for API scoping.

## Session state

After login, the encrypted session cookie holds:

- User id (Doido auth)
- **Active company id** (`company_id`)
- **Active prestador id** (`empresa_id` — `empresas.id`)

`POST /tenant/company` with `{ "company_id": N }` validates membership and sets the tenant company (auto-selects prestador when the company has exactly one).

`POST /tenant/prestador` with `{ "empresa_id": N }` sets the active prestador; the record must belong to the active company.

## API enforcement

Tenant-scoped controllers call `require_tenant`, which:

1. Requires authenticated user
2. Requires active company and prestador in session
3. Loads `TenantAccess { user, company, prestador, papel }`

Fiscal data is filtered by `prestador.id` (`empresa_id` on rows) via `tenant_scope::*` helpers.

## Frontend

1. Login → `/tenant` if no `company_ativa` in `/api/me`
2. Select company → `/tenant/prestador` if no `prestador_ativo`
3. Select prestador → `/app`

## Testing

[`backend/tests/tenant_api_test.rs`](../../backend/tests/tenant_api_test.rs) covers sign-in, company/prestador switch, and scoped cliente access.

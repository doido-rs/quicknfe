# Multi-tenant model

## Concepts

- **User** — authenticates via Doido (`users` table).
- **Empresa** — fiscal entity (CNPJ, IE, municipality, etc.).
- **Membership** — `empresas_usuarios` links users to empresas with a `papel` (role).

## Session state

After login, the encrypted session cookie holds:

- User id (Doido auth)
- **Active empresa id** (app key `empresa_id` in tenant module)

`POST /tenant/empresa` with `{ "empresa_id": N }` validates membership then sets active tenant.

## API enforcement

Tenant-scoped controllers call `require_tenant`, which:

1. Requires authenticated user
2. Requires active empresa in session
3. Loads `TenantAccess { user, empresa, papel }`

Queries filter by `empresa_id` or use `tenant_scope::*` helpers so cross-tenant access returns 404/403.

## Frontend

1. Login → `/tenant` if no `empresa_ativa` in `/api/me`
2. Select empresa → redirect `/app`
3. Sidebar shows resources; all API calls send cookies (`credentials: 'include'`)

## Testing

[`backend/tests/tenant_api_test.rs`](../../backend/tests/tenant_api_test.rs) covers sign-in, tenant switch, and scoped cliente access.

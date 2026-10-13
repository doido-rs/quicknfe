# QuickNFe — agent guide

This document describes what the system implements today, how the **resource-driven UI** works, and how **integrations** (CNC, NFSe/DPS, certificates) fit into user and API workflows. Use it when extending features, debugging forms, or aligning backend and frontend behavior.

## Product scope

QuickNFe is a multi-tenant web app for **NFSe (Nota Fiscal de Serviço eletrônica)** lifecycle management in Brazil:

- Cadastros: prestadores (emitters), tomadores (`clientes`), intermediários, municípios (IBGE), CNA (código tributação nacional).
- **Notas fiscais** (`documentos`): structured form aligned with DPS/NFSe data (competência, partes, municípios, valores, ISSQN, PIS/COFINS federal panel).
- **Itens** per nota, **certificados digitais** per prestador, **lotes** and **logs de retorno (SEFAZ)**.
- Outbound **CNC** (cadastro nacional consult) and **SEFIN Nacional** transmission (backend), with local mock/dev paths.

Stack: Rust/Doido backend (MySQL), React/Vite frontend (Bun), TanStack Query/Router, shadcn/ui. See `docs/architecture/stack.md` and `docs/README.md` for harness, CI, MCP/OAuth notes.

---

## Multi-tenant and session workflow

1. **Auth**: email/password via Doido (`/users/sign_in`, `/users/sign_up`, `/users/sign_out`). Session cookie; frontend uses `credentials: 'include'`.
2. **`GET /api/me`**: returns user, companies, active company, prestadores list, **active prestador** (`prestador_ativo`), and role (`papel`).
3. **Tenant gates** (`AppLayout`):
   - No session → `/login`.
   - No active company → `/tenant` (company picker; `POST /tenant/company`).
   - No active prestador → `/tenant/prestador` (`POST /tenant/prestador`).
4. **API scoping**: authenticated routes use `require_tenant`; documentos/clientes/intermediários are scoped to the active **company** and prestador **empresa_id** where applicable (`tenant_scope`).

Active prestador drives defaults on new notas (pre-fill `empresa_id`, fetch prestador município for emissão/prestação).

---

## Resource-driven UI (single source of truth)

All CRUD screens are generated from `frontend/src/resources/definitions.ts`:

| Concept | Location / behavior |
|--------|------------------------|
| `ResourceDef` | One entry per API resource: `key`, `title`, `path`, `fields`, optional `formSections`, `nested`, `cncLookup`, flags |
| Routes | `/app/:resourceKey`, `/new`, `/:id`, `/:id/edit` via `ResourceLayout` + resource pages |
| Sidebar | `sidebarResources()` — `menuPlacement` defaults to `sidebar` |
| Settings hub | `/app/configuracoes` — `settingsResources()` (`menuPlacement: 'settings'`: CNA, municípios) |
| List | `ResourceListPage` — infinite query, `page_size` / `page_token`; **documentos** use `DocumentoListTable` |
| Show | Generic `ResourceShowPage` except **documentos** → `DocumentoShowView` (read-only form layout) |
| New / Edit | `ResourceNewPage` / `ResourceEditPage`; documentos use `buildDocumentoBody` / `flattenDocumentoRowToForm` |

### Registered resources (summary)

| `key` | Menu | API base | Notes |
|-------|------|----------|--------|
| `documentos` | Nota fiscal | `/api/documentos` | Custom list/show; nested itens + logs; assinar/XML |
| `lotes` | Nota fiscal | `/api/lotes` | Transmission batches |
| `logs-retorno` | (no menu) | `/api/logs-retorno` | Global log index; also nested under documento |
| `prestadores` | Cadastros | `/api/prestadores` | CNC lookup; nested certificados |
| `clientes` | Cadastros | `/api/clientes` | Tomadores; CNC lookup; scoped by prestador |
| `intermediarios` | Cadastros | `/api/intermediarios` | Scoped by prestador |
| `municipios` | Configurações | `/api/municipios` | Read-only UI (no create/delete) |
| `cnas` | Configurações | `/api/cnas` | Read-only UI |

---

## Forms — base behavior

Forms are **not** hand-written per screen. They share:

- **State**: `react-hook-form` via `useResourceFieldForm(def.fields)` — all values are **strings** in the form; conversion happens in `buildBody` / resource-specific builders.
- **Rendering**: `ResourceFormFields` + `ResourceFieldInput`.
- **Submit pipeline (generic)**:
  1. `buildBody(fields, values)` — skips `readOnly`, dotted keys, and `file` fields; maps booleans, numbers, `*_id` to numbers; empty string → `null`.
  2. `POST` or `PATCH` to `def.path`.
  3. TanStack Query invalidation; **Sonner** toasts (`toastSuccess` / `toastFromError`).

### `FieldDef` capabilities

| Property | Purpose |
|----------|---------|
| `type` | `text`, `number`, `date`, `textarea`, `relation`, `enum`, `file`, `boolean` |
| `required` | HTML/RHF validation on relation and inputs |
| `defaultValue` | Initial string in `emptyForm` |
| `readOnly` | Excluded from API body; shown disabled or as text |
| `formOnly` | Used in forms; may still be sent if included in `buildBody` filter rules |
| `relationResource` | Enables `ForeignKeyAutocomplete` (debounced search, pagination) |
| `relationScopeFromField` | Passes `empresa_id` query param on list (tomador/intermediário vs prestador) |
| `relationCopyTo` | On select, copy fields from related row into form |
| `enumOptions` | Select + read-only label resolution |
| `fileUploadAs` | Base64 + filename keys for API (certificados) |
| `accept` | File input accept list |

### `FormSectionDef` (layout only)

Fields remain in flat `fields[]` for API mapping; sections control UX:

| Section `type` | Behavior |
|----------------|----------|
| `identificacao-nota` | Competência, série/número, checkbox “informar manualmente” (inverts `serie_numero_automatico`) |
| `prestador-partes-row` | Prestador + tomador/intermediário; optional `partesLocalidade: true` → `ParteLocalidadeFields` |
| `municipios-nota` | Emissão vs prestação município pair |
| `fields` | Titled grid of `fieldKeys`, `columns` 1 or 2 |

**Read-only mode** (`readOnly` on `ResourceFormFields`): no submit actions; labels + static text; relations use `relationDisplayLabels` / `readOnlyFieldDisplay` (documento show enriches names from API).

**Actions**: `submitLabel`, `cancelLink`, `actionsAlign: 'end'` (notas fiscais align Criar/Cancelar to the right).

### Unsaved changes — required behavior (not implemented yet)

**Product rule (all editable forms):** the user must **not** leave the form—with unsaved edits—by closing the flow, using Cancel/back links, sidebar navigation, in-app route changes, or browser tab close/reload—unless they **save successfully** or **explicitly confirm discarding** changes.

**Status:** documented requirement only; **not implemented in the codebase today**. New work should plan for this guard rather than adding one-off bypasses.

**Scope — every form that mutates data:**

| Surface | Examples |
|---------|----------|
| Resource create | `ResourceNewPage` + `ResourceFormFields` |
| Resource edit | `ResourceEditPage` + `ResourceFormFields` |
| Nested create | `NestedResourcePanel` inline POST forms (itens, certificados, …) |
| Documento draft itens | `DocumentoItensDraftPanel` on new nota (local draft counts as dirty until create completes or user clears items) |

**Out of scope:** read-only views (`DocumentoShowView`, generic show without edit form, list pages).

**Expected UX when implemented:**

1. **Dirty detection** — compare current values to last saved baseline (e.g. `react-hook-form` `formState.isDirty` after `reset` from server defaults on edit; empty defaults on create). Include nested/draft panels in the same dirty signal where they hold unpublished data.
2. **In-app navigation** — intercept TanStack Router transitions (sidebar, “Voltar”, list links, breadcrumbs) and show a confirm dialog: stay, save (if applicable), or discard and leave.
3. **Cancel actions** — “Cancelar” / back links follow the same rule (confirm if dirty), not silent navigation.
4. **Browser leave** — optional `beforeunload` prompt when dirty (standard browser wording).
5. **After successful submit** — clear dirty state (`reset` with new baseline) then allow navigation (including post-create redirect to show).

**Implementation notes for agents (future):**

- Prefer a **shared hook or wrapper** (e.g. on `ResourceFormFields` / resource pages) so all forms behave consistently; avoid per-resource copy-paste.
- Do not block navigation when the form is pristine or when only CNC consult ran without editing fields (consult fills fields → that should mark dirty).
- Successful nested item POST clears only that nested form’s dirty slice; parent edit form dirty state unchanged unless parent fields were edited.

### Relation fields (`ForeignKeyAutocomplete`)

- Resolves API path via `relationApiPath(resourceKey)` → e.g. `/api/clientes`.
- Lists with `page_token`, `page_size`, optional `q`, optional **`empresaId`** when scoped.
- Selected row: optional GET by id for display label (`relationOptionLabel`).
- Documento show: API returns `prestador_razao_social`, `tomador_razao_social`, `intermediario_razao_social`, `municipio_emissao`, `municipio_prestacao` — mapped in `buildDocumentoRelationLabels`.

### Nested resources (`NestedDef`)

Shown on **show** (and logs read-only) via `NestedResourcePanel`:

- List: `GET parentPath?page_size=…`
- Create: inline form → `POST parentPath` (unless `allowCreate: false`)
- **`prepareCreateBody`**: e.g. itens copy `descricao` from `descricao_servico` before POST
- File nested fields use `buildCreateBodyWithFiles` (base64)

**Nota fiscal — create flow**: `ResourceNewPage` adds `DocumentoItensDraftPanel` (local draft). On **Criar**:

1. `buildDocumentoBody` + optional `applyItensToDocumentoBody` (sum item `valor_total_bruto` → valor operação/líquido).
2. `POST /api/documentos`.
3. Sequential `POST /api/documentos/{id}/itens` for each draft item.
4. Navigate to show.

---

## Nota fiscal — form-specific logic

### Tomador / intermediário localidade

`documentoParteLocalidadeFields()` adds per-parte:

- `tomador_localidade` / `intermediario_localidade`: `brasil` | `exterior` | `nao_informado`
- **Brasil**: relation `cliente_id` / `intermediario_id` (scoped by `empresa_id`).
- **Exterior**: flat `*_exterior_*` fields → JSON `tomador_exterior` / `intermediario_exterior` on API.
- **Não informado**: clears ids and exterior objects.

`buildDocumentoBody` (frontend) and `normalizar_partes` (backend) must stay aligned.

### Hidden / defaulted API fields

`buildDocumentoBody` sets when missing:

- `modelo` → `NFSe`
- `data_emissao` → now ISO if empty
- `ambiente` → `2`, `tipo_emissao` → `1`, `status` → `novo`
- ISSQN / PIS-COFINS defaults (`trib_issqn`, `reg_esp_trib_issqn`, `cst_pis_cofins`, `tp_ret_pis_cofins_csll`, boolean ISSQN flags)
- **`valor_liquido`** = operação − descontos incondicionado − condicionado

Backend **`normalizar_valores_documento`**: requires **positive** `valor_total_servicos_produtos`.

### Série/número automáticos

When `serie_numero_automatico === true`, frontend sends `numero_documento: 0`, `serie: ''`; backend `preparar_identificacao` assigns next number ( série `AUTO` ).

### New documento UX defaults

- Prestador = `me.prestador_ativo.id`
- `data_competencia` = today
- After prestador load: `municipio_emissao_id` and `municipio_prestacao_id` from prestador’s município

### List and show (documentos)

- **List** (`DocumentoListTable`): data (competência/emissão), prestador, tomador, intermediário, valor, situação; API list DTO adds `*_razao_social` (`documento_presentacao.rs`).
- **Show** (`DocumentoShowView`): same sections as edit, read-only; **Assinar nota fiscal**, **Baixar XML**, Editar; nested itens + logs SEFAZ.

---

## Integration fields and workflows

### 1. CNC — Consulta Cadastro (`cncLookup`)

**Purpose**: Prefill prestador/tomador from Receita/cadastro nacional (CNC).

**Where**: `ResourceDef.cncLookup` on `prestadores` and `clientes`.

**UI**: `CncConsultaField` at top of form (only when not read-only). User fills CPF/CNPJ (and optionally município for IBGE hint), clicks **Consultar CNC**.

**Frontend workflow**:

1. Validate 11 or 14 digits on `inscricaoFieldKey`.
2. If `municipioFieldKey` set, resolve `codigo_ibge` via `GET /api/municipios/{id}`.
3. `GET /api/cnc/cad?inscricao_federal=&cod_municipio=&empresa_id=` (`consultaCncCad`).
4. On success:
   - 0 itens → message / toast
   - 1 item → `applyCncItemToForm` (razão social, IM, email, telefone, `municipio_id` if returned)
   - Multiple → user picks a row (IM / IBGE shown)

**Backend workflow** (`ApiCncController::consulta_cad`):

1. Tenant + prestador scope (`empresa_id` query or active prestador).
2. **mTLS credentials**: latest certificado for prestador (or `certificado_id`), unless CNC base URL is localhost/127.0.0.1 (mock — no cert).
3. Proxy to CNC service; map to public DTO (`resposta_publica`).

**Field mapping** (`CncLookupDef`):

| Key | Typical target |
|-----|----------------|
| `inscricaoFieldKey` | `cpf_cnpj` |
| `razaoSocialFieldKey` | `razao_social` |
| `inscricaoMunicipalFieldKey` | `inscricao_municipal` (prestador) |
| `emailFieldKey` / `telefoneFieldKey` | tomador |
| `municipioFieldKey` | `municipio_id` |

### 2. Certificado digital (nested under prestador)

**Purpose**: Store A1 PFX for mTLS (CNC, NFSe transmission).

**UI**: Nested panel on prestador **show** — file field `.pfx,.p12` → `arquivo_base64` + `arquivo_nome`, senha, validade.

**API**: `/api/prestadores/{prestador_id}/certificados` CRUD.

**Workflow**: Upload on show/edit prestador after prestador exists; CNC in **dev mock** may work without cert; production CNC/SEFIN require valid cert on prestador.

### 3. NFSe / DPS — XML, assinatura, download, transmissão

| Step | API | UI today | Notes |
|------|-----|----------|--------|
| Montar rascunho XML | — | — | `montar_dps_xml(documento)` server-side |
| Assinar | `POST /api/documentos/{id}/assinar` | **Assinar nota fiscal** | Inserts XML-DSig stub into `xml_assinado` |
| Download | `GET /api/documentos/{id}/xml` | **Baixar XML** | Priority: `xml_processado` → `xml_assinado` → rascunho DPS |
| Transmitir SEFIN | `POST /api/documentos/{id}/transmitir` | *Not wired in frontend* | mTLS + optional `certificado_id`; updates doc/logs |

Transmission implementation lives in `backend/app/services/nfse/` (ambiente URLs, resposta parsing, logs).

### 4. Itens da nota

- Fields: número, descrição serviço, **CNA** relation, quantidade, valores.
- CNA list is read-only catalog; item create stores `codigo_tributacao_nacional` from CNA row.
- Optional nested **tributos** per item exist in API (`/api/itens/{item_id}/tributos`) — not exposed as main UI resource in definitions.

### 5. Logs de retorno (SEFAZ)

- Nested under documento: **read-only** (`allowCreate: false`).
- Populated by transmission/processing backend paths; display código, mensagem, correção, data.

---

## Pagination and API client

- **Cursor pagination**: `page_token`, `page_size` (default 20, max 100). Query params deserialized flexibly (string or number) in `pagination.rs`.
- **Frontend**: `resourceApi.listPage`, infinite scroll “Carregar mais” on lists.
- **Base URL**: `VITE_API_URL` + path; JSON except XML download (blob).

---

## UI infrastructure

- **Page titles**: `PageTitle` + `PageTitleProvider`; screens set title (e.g. resource name, “Novo — …”).
- **Toasts**: Sonner in `AppLayout`; mutations use `toastSuccess` / `toastFromError`.
- **Theme**: light/dark toggle in top bar.
- **Delete**: list tables → confirm dialog → `DELETE` + toast.

---

## Backend API map (authenticated)

Core routes in `backend/config/routes.rs`:

- Session/tenant: `/api/me`, `/tenant/company`, `/tenant/prestador`, `/api/dashboard`
- CRUD: prestadores, clientes, intermediários, lotes, documentos
- Documento actions: `…/xml`, `…/assinar`, `…/transmitir`
- Nested: `documentos/{doc_id}/itens`, `eventos`, `logs-retorno`
- Itens tributos, global logs-retorno
- Reference: municipios, cnas (index/show)
- Integration: **`GET /api/cnc/cad`**
- Certificados under prestadores

OpenAPI: `docs/api/openapi.yaml` (may lag newest routes like `assinar` — verify routes.rs when in doubt).

---

## Conventions for agents

1. **Prefer extending `definitions.ts`** over one-off forms when adding CRUD fields or sections.
2. **Documento changes**: update both `buildDocumentoBody` / `flattenDocumentoRowToForm` and backend `DocumentoForm` + normalizers.
3. **CNC**: keep mock-friendly behavior (localhost CNC URL skips mTLS); do not break `applyCncItemToForm` mapping.
4. **Scoped relations**: tomador/intermediário must keep `relationScopeFromField: 'empresa_id'`.
5. **Tests**: frontend Vitest (`frontend/src/**/*.test.ts`, `ResourceCrud.test.tsx`); backend `cargo test`; integration `make test-integration` (MySQL).
6. **Minimal diffs**: match existing patterns (`NestedResourcePanel`, `buildNestedCreateBody`, resource pages).
7. **Unsaved changes (planned)**: all editable forms must eventually guard navigation when dirty; see [Unsaved changes — required behavior (not implemented yet)](#unsaved-changes--required-behavior-not-implemented-yet). Do not implement ad-hoc “leave anyway” without product alignment.

---

## Key file index

| Area | Files |
|------|--------|
| Resource definitions | `frontend/src/resources/definitions.ts` |
| Form core | `frontend/src/lib/resourceForm.ts`, `ResourceFormFields.tsx`, `ResourceFieldInput.tsx` |
| Documento | `documentoParteLocalidade.ts`, `documentoItens.ts`, `DocumentoShowView.tsx`, `DocumentoListTable.tsx`, `DocumentoItensDraftPanel.tsx` |
| CNC UI/API | `CncConsultaField.tsx`, `lib/cncConsulta.ts`, `api/cnc.ts`, `ApiCncController` |
| Documento API | `documentos_controller.rs`, `documento_presentacao.rs`, `nfse/dps.rs` |
| Routes | `backend/config/routes.rs` |
| Tenant | `backend/app/auth/tenant.rs`, `tenant_scope.rs`, `AuthContext.tsx` |

---

*Last aligned with codebase features: multi-tenant NFSe CRUD, CNC consult, documento list/show/sign/XML, draft itens on create, tributação municipal/federal form sections, cursor pagination, Sonner toasts. Planned: global unsaved-changes navigation guard on all forms (documented, not built).*

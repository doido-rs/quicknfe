# QuickNFe — agent guide

This document describes what the system implements today, how the **resource-driven UI** works, and how **integrations** (CNC, NFSe/DPS, certificates) fit into user and API workflows. Use it when extending features, debugging forms, or aligning backend and frontend behavior.

## Product scope

QuickNFe is a multi-tenant web app for **NFSe (Nota Fiscal de Serviço eletrônica)** lifecycle management in Brazil:

- Cadastros: prestadores (emitters), tomadores (`clientes`), intermediários, municípios (IBGE), CNA (código tributação nacional).
- **Notas fiscais** (`documentos`): structured form aligned with the **DPS XML layout** in `layout_dps_leaiute.csv` + `layout_dps.csv` (competência, partes, municípios, valores, ISSQN, PIS/COFINS federal panel, itens).
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
| `type: 'enum'` | Fixed set via `enumOptions` (`{ value, label }[]`); renders a **select**, not free text. Example: certificado nested `tipo` — see [Certificado digital](#2-certificado-digital-nested-under-prestador). |
| `type: 'date'` | Calendar dates only (competência, emissão, validade certificado, `data_registro`, …). **Display and edit** must follow the user’s [date format preference](#user-preferences--date-format); wire/API values stay ISO — see that section. |
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

### Unsaved changes — required behavior

**Product rule (all editable forms):** the user must **not** leave the form—with unsaved edits—by closing the flow, using Cancel/back links, sidebar navigation, in-app route changes, or browser tab close/reload—unless they **save successfully** or **explicitly confirm discarding** changes.

**Status:** implemented via `UnsavedChangesProvider`, `useUnsavedFormGuard`, TanStack Router `useBlocker`, and `GuardedLink` on resource create/edit flows (including nested panels and documento draft itens).

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
2. **In-app navigation** — intercept TanStack Router transitions (sidebar, “Voltar”, list links, breadcrumbs) and show a confirm dialog: stay, **Salvar** (when edit page registered a save handler), or discard and leave.
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
- **Show** (`DocumentoShowView`): same sections as edit, read-only; **Assinar nota fiscal**, **Transmitir SEFIN**, **Baixar XML**, Editar; nested itens (+ tributos) + logs SEFAZ.

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

**Purpose**: Store digital certificate files (typically **A1** PFX) for mTLS (CNC, NFSe transmission) and XML-DSig.

**UI**: Nested panel on prestador **show** (`NestedResourcePanel` on prestador show).

| Field | `FieldDef` | Notes |
|-------|------------|--------|
| `tipo` | **`type: 'enum'`** + `enumOptions` | **Must be a select**, not a plain text input. Allowed values match DB check constraint: `A1`, `A3`, `PFX`, `PEM`. Default UX: preselect `A1` when creating (most uploads are A1 `.pfx`/`.p12`). |
| `arquivo` | `type: 'file'`, `accept: '.pfx,.p12'`, `fileUploadAs` | POST body: `arquivo_base64` + `arquivo_nome` |
| `senha` | `text` (password in UI) | Encrypted server-side |
| `data_validade` | `type: 'date'` | User [date format](#user-preferences--date-format); API `YYYY-MM-DD` |

**API**: `/api/prestadores/{prestador_id}/certificados` CRUD — `tipo` is a string enum on the wire (one of the values above).

**Workflow**: Upload on show/edit prestador after prestador exists; CNC in **dev mock** may work without cert; **assinar DPS**, production CNC, and SEFIN require valid cert on prestador (see [Certificado para assinatura XML](#certificado-para-assinatura-xml)).

### 3. NFSe / DPS — XML, assinatura, download, transmissão

**Schema authority:** the DPS XML tree and rules are defined by **`layout_dps_leaiute.csv`** (structure) and **`layout_dps.csv`** (business/reception rules). See [DPS XML layout (CSV)](#dps-xml-layout-csv) for setup, logical groups, and how UI/DB map into XML.

| Step | API | UI today | Notes |
|------|-----|----------|--------|
| Montar rascunho XML | — | — | `montar_dps_xml_context` (identificação, prestador, partes, serviço, valores/tributos); cobertura obrigatória `infDPS` rastreada em `layout_cobertura_parcial` |
| Validar DPS | — (before assinar/transmitir) | — | **Product rule:** every **mandatory** `infDPS` field from **`layout_dps_leaiute.csv`** must pass **presence** validation; conditional rules from **`layout_dps.csv`** — see [Mandatory XML field presence](#mandatory-xml-field-presence) |
| Assinar | `POST /api/documentos/{id}/assinar` | **Assinar nota fiscal** | XML-DSig enveloped com **certificado A1 PFX** do prestador — **obrigatório**; ver [Certificado para assinatura XML](#certificado-para-assinatura-xml) |
| Download | `GET /api/documentos/{id}/xml` | **Baixar XML** | Priority: `xml_processado` → `xml_assinado` → rascunho DPS |
| Transmitir SEFIN | `POST /api/documentos/{id}/transmitir` | **Transmitir SEFIN** on documento show | mTLS + optional `certificado_id`; updates doc/logs |

Transmission implementation lives in `backend/app/services/nfse/` (ambiente URLs, resposta parsing, logs).

### Mandatory XML field presence

**Product rule:** before **Assinar nota fiscal** or **Transmitir SEFIN**, the backend must validate that **all required DPS XML fields** have a present, non-empty value in the data that backs the XML (and that the builder can emit the corresponding node when the field is mandatory in the leiaute).

| Source of “required” | Meaning |
|----------------------|---------|
| **`layout_dps_leaiute.csv`** (leiaute) | Rows under `NFSe/infNFSe/DPS/infDPS/` with `OCOR.` **1-1**, **1-n**, or **1..n** on elements **`E`**, **`A`**, or **`ID`** (see `dps_layout::mandatory_infdps_fields` or CSV loader equivalent) |
| **`layout_dps.csv`** (RN) | Applicability **Obrig.** / effect **Rej.** (and related columns) per `CAMINHO NO XML` / `CAMPO`; tomador exterior, exportação, recepção SEFIN, etc. |

**Validation expectations for agents:**

1. **Presence, not optional omission** — empty string, missing relation, zero items when `1-n` serviço lines are required, or absent JSON for exterior parties must **fail** with `ApiError::Validation` (clear message referencing layout path or form field where possible).
2. **Single pipeline** — run the same checks from `validar_dps_contexto` (and extensions) on **`POST …/assinar`** and **`POST …/transmitir`** after `load_dps_context`; do not sign or transmit when mandatory data is missing.
3. **Grow toward full leiaute coverage** — today `validar_dps_contexto` covers critical RN/form rules and `layout_cobertura_parcial` tracks XML tag emission gaps; **target state** is mandatory **presence** for every leiaute-required field that QuickNFe owns (DB, computed, or nested itens), aligned with `montar_dps_xml_context`.
4. **Tests** — add/adjust cases in `nfse/rn_validacao.rs`, `nfse/dps.rs` (`layout_cobertura_parcial`, golden tags), and documento API tests when new mandatory fields are mapped.

### Certificado para assinatura XML

**Product rule:** **Assinar nota fiscal** requires a **valid digital certificate** on the active prestador (nested **certificados digitais** — typically **A1** `.pfx`/`.p12` with senha and `data_validade`).

| Requirement | Detail |
|-------------|--------|
| **When** | Every successful **`POST /api/documentos/{id}/assinar`** must produce **real XML-DSig** (enveloped signature over `infDPS`) using the prestador’s PFX private key — not an empty `<Signature/>` placeholder. |
| **Which cert** | Latest certificado for the documento’s `empresa_id` (prestador), or explicit `certificado_id` when the API supports choosing among multiples (same pattern as transmissão). |
| **Failure modes** | No cert on prestador, unreadable file, wrong senha, or expired cert → **4xx validation** with actionable message; UI should direct the user to upload certificado on prestador show. |
| **Dev vs prod** | Local **CNC mock** (localhost) may omit mTLS; **signing and SEFIN** still require a real cert for homologation/production paths. Do not treat “stub signature” as acceptable for flows that reach SEFIN. |

Implementation: `ultimo_certificado_empresa` + `assinar_dps_xml_rascunho` in `nfse/assinatura.rs` (`documentos_controller::assinar`).

### 4. Itens da nota

- Fields: número, descrição serviço, **CNA** relation, quantidade, valores.
- CNA list is read-only catalog; item create stores `codigo_tributacao_nacional` from CNA row.
- Nested **tributos** per item: API `/api/itens/{item_id}/tributos`; UI expandível em **Itens da nota** no show da nota (`ItemTributosPanel`).

### 5. Logs de retorno (SEFAZ)

- Nested under documento: **read-only** (`allowCreate: false`).
- Populated by transmission/processing backend paths; display código, mensagem, correção, data.

---

## DPS XML layout (CSV)

### Role of the CSV files

**The canonical definition of DPS XML in this repository is two CSV files at the repo root** (NFS-e Nacional — Declaração de Prestação de Serviços):

| File | Purpose |
|------|---------|
| **`layout_dps_leaiute.csv`** | **XML leiaute (structure)** — one row per field under the national tree: `CAMINHO NO XML`, `CAMPO`, `ELE`, `TIPO`, `OCOR.` (cardinality), `TAM.`, `DESCRIÇÃO`, notas. Use for tag names, hierarchy, order, types, and **mandatory presence** (`OCOR.` 1-1 / 1-n / 1..n on `E` / `A` / `ID`). Primary input for `montar_dps_xml` and coverage tests (`layout_cobertura_parcial`). |
| **`layout_dps.csv`** | **Business and reception rules (RN)** — rules keyed by `CAMINHO NO XML` and/or `CAMPO`, with applicability (`APLIC.`), effect (`EFEITO`), `CÓD. ERRO`, `MSG. ERRO`, rule level, and emissor/reception columns. Use for server-side validation in `validar_dps_contexto` and alignment with SEFIN reject codes in `logs-retorno`. |

Together they replace the former monolithic **`layout_dps.xlsx`** workflow: **structure in leiaute CSV, constraints in `layout_dps.csv`**.

- **Do not invent DPS tags** without a matching row (or explicit group) in **`layout_dps_leaiute.csv`**.
- **UI form sections**, **`DocumentoForm` / DB columns**, **item (serviço) fields**, **`montar_dps_xml`**, and **validators** must stay aligned with these CSVs over time.
- When the national layout version changes, **refresh the CSVs first** (export from the official SNNFSe workbook if needed), then trace impacts through migrations, forms, and XML builder.
- Upstream **`.xlsx`** files are optional **sources for regenerating CSVs only** — they are not authoritative in git unless explicitly re-exported into the two CSV paths below.

**Repository location (authoritative):**

- `/layout_dps_leaiute.csv` — leiaute / XML schema rows (filename uses `leaiute`; treat as *leiaute*).
- `/layout_dps.csv` — RN and reception rules (and any national rows not stored in the leiaute file).

### National workbook content (mapped to CSV)

The national package may still ship multiple worksheets; map them into the two CSVs when exporting:

| National worksheet (source) | Target in repo |
|----------------------------|----------------|
| **`LEIAUTE DPS_NFS-e`** | **`layout_dps_leaiute.csv`** — full structural leiaute including `NFSe/…` and `…/infDPS/…` paths |
| **`RN DPS_NFS-e`**, **`RN_RECEPCAO_DPS`** | **`layout_dps.csv`** — business and reception rules |
| **`MUN.INCID_INFO.SERV.`**, **`EXPORTACAO_EMISSÃO_NFS-e`**, CNA/LI/export rows | Either CSV row set that owns that concern, or dedicated sections within **`layout_dps.csv`** — document the mapping in commit messages when exporting |

**Expected leiaute header row (`layout_dps_leaiute.csv`):** `#`, `CAMINHO NO XML`, `CAMPO`, `ELE`, `TIPO`, `OCOR.`, `TAM.`, `DESCRIÇÃO`, `NOTAS EXPLICATIVAS`.

**Typical RN header row (`layout_dps.csv`):** `#`, `CAMINHO NO XML`, `CAMPO`, rule description columns, `APLIC.`, `EFEITO`, `CÓD. ERRO`, `MSG. ERRO`, nível da regra, emissor/reception flags.

### How to read and use the CSVs (setup)

When implementing or reviewing DPS mapping:

1. Open **`layout_dps_leaiute.csv`** and filter paths under **`NFSe/infNFSe/DPS/infDPS/`** for emission scope (QuickNFe builds standalone **`DPS`** / `infDPS`; reconcile paths with `backend/app/services/nfse/dps.rs`).
2. Build or verify XML against leiaute rows (**path + campo + `OCOR.` + `TIPO`**).
3. Cross **`layout_dps.csv`** for **Obrig./Rej.** rules and error codes before assinar/transmitir.
4. For **CNA / serviço / LI / grupos específicos**, use RN rows referencing `cTribNac`, `MUN.INCID_INFO.SERV.`, etc., plus app **itens** (`cna_id`, `descricao_servico`) and the **CNA** catalog.
5. For **tomador/intermediário exterior** and export ISSQN, use RN/export rows with `tomador_localidade` / `intermediario_localidade` and tributação ISSQN form fields.
6. For each leiaute row, record **QuickNFe data source** (DB column, relation, item field, or computed).
7. Cross-check **namespace and root element**: today `montar_dps_xml` uses root **`DPS`**, namespace **`http://www.gov.br/nfse`**, wrapper **`infDPS`** with `Id` (see `backend/app/services/nfse/dps.rs`). National paths in CSV often prefix `NFSe/infNFSe/DPS/…` — keep reconciliation documented.
8. Extend **`montar_dps_xml`** to emit **all mandatory leiaute nodes**, loading prestador, tomador/intermediário, municípios IBGE, itens + CNA as required.

**Backend loaders:** `dps_layout.rs` and `dps_rn.rs` may still read **`layout_dps.xlsx`** until migrated — **new work and tests must treat the repo-root CSVs as source of truth**; point loaders at `layout_dps_leaiute.csv` / `layout_dps.csv` when touching those modules.

**Current implementation gap:** `montar_dps_xml_context` serializa os grupos principais; ainda faltam campos obrigatórios do leiaute `infDPS` (cobertura parcial — teste `layout_cobertura_parcial` em `nfse/dps.rs`).

**RN / recepção:** critical checks in `validar_dps_contexto` before assinar/transmitir, fed from **`layout_dps.csv`** (today via `dps_rn.rs` + xlsx). **Mandatory leiaute fields** must be **presence-validated** per [Mandatory XML field presence](#mandatory-xml-field-presence) using **`layout_dps_leaiute.csv`** as coverage grows.

### Runtime configuration (SEFIN / ambiente)

These settings must stay consistent with **`tpAmb`** on each documento (field `ambiente`, default `2` in forms):

| Setting | Where | Purpose |
|---------|--------|---------|
| `settings.sefin.ambiente` | `backend/config/development.yml`, `production.yml`, `test.yml` | Global SEFIN mode: **1** produção, **2** homologação / produção restrita |
| `settings.sefin.base_url` | Same; override with env **`SEFIN_BASE_URL`** | SEFIN Nacional API base (no trailing slash) |
| `settings.cnc.base_url` | Same; override with **`CNC_BASE_URL`** | CNC consulta (localhost mock skips mTLS) |
| Documento `ambiente` | DB / `DocumentoForm` | Must match `settings.sefin.ambiente` for transmission (`AmbienteNfse::ensure_matches_config`) |
| Documento `tipo_emissao` | DB / form default `1` | Emission type per national rules (see `layout_dps_leaiute.csv`) |
| Certificado A1 | Nested under prestador | mTLS for CNC (non-mock) and SEFIN transmission |

**Development defaults:**

- Homologation API: `https://sefin.producaorestrita.nfse.gov.br/API/SefinNacional` (or local mock via `SEFIN_BASE_URL`).
- CNC: `https://adn.producaorestrita.nfse.gov.br/cnc/consulta` or `http://127.0.0.1:8090/cnc/consulta` for mock.

**Endpoints derived in code** (`ambiente.rs`): `{base_url}/nfse` (emissão), `{base_url}/dps/{id}` (consulta DPS).

### Full logical DPS layout ↔ QuickNFe mapping

Use this as the **checklist** when completing XML generation against **`layout_dps_leaiute.csv`**. Exact tag names and nesting must match the leiaute CSV row-for-row; names here follow common NFS-e Nacional groupings and link to existing app data.

| Layout group (logical) | Primary XML area (per `layout_dps_leaiute.csv`) | QuickNFe source |
|--------------------------|-------------------------------------------|-----------------|
| **Identificação DPS** | `infDPS`: ambiente, emissão, competência, série, número, tipo emissão, etc. | `documentos_fiscais`: `ambiente`, `data_emissao`, `data_competencia`, `serie`, `numero_documento`, `tipo_emissao`, `modelo`; auto série/número via `serie_numero_automatico` |
| **Prestador** | Dados do emitente/prestador de serviço | `empresa_id` → `empresas` / prestador: `cpf_cnpj`, `razao_social`, `inscricao_municipal`, `regime_tributario`, município IBGE |
| **Tomador** | Tomador do serviço (Brasil, exterior ou omitido) | `tomador_localidade` + `cliente_id` → `clientes`, or `tomador_exterior` JSON; CNC may prefill cadastro |
| **Intermediário** | Intermediário (opcional) | `intermediario_localidade` + `intermediario_id` or `intermediario_exterior` |
| **Localidade / município** | Município emissão e prestação | `municipio_emissao_id`, `municipio_prestacao_id` → `municipios.codigo_ibge`, nome, UF |
| **Serviço / itens** | Discriminação do serviço (1..n linhas) | Nested **itens**: `numero_item`, `descricao` / `descricao_servico`, `cna_id` → `codigo_tributacao_nacional`, `quantidade`, `valor_unitario`, `valor_total_bruto` |
| **Valores da operação** | Totais, descontos, líquido, valor intermediário | `valor_total_servicos_produtos`, `desconto_incondicionado`, `desconto_condicionado`, `valor_liquido`, `valor_recebido_intermediario`; draft itens on create feed totals via `applyItensToDocumentoBody` |
| **Tributação municipal (ISSQN)** | ISSQN, exigibilidade, retenção, benefícios, BC/alíquota/valor | Form section *Tributação Municipal*: `trib_issqn`, `reg_esp_trib_issqn`, flags `issqn_*`, `issqn_aliquota`, `issqn_base_calculo`, `issqn_valor` |
| **Tributação federal** | PIS/COFINS/CSLL, retenções federais | Form section *Tributação Federal*: `cst_pis_cofins`, `tp_ret_pis_cofins_csll`, `valor_irrf`, `valor_contrib_sociais_retidas`, `valor_contrib_previdenciaria_retida` |
| **Informações complementares** | Campos adicionais / B2B / RT | `numero_doc_responsabilidade_tecnica`, `documento_referencia`, `informacoes_complementares`, `numero_pedido_ordem_b2b` |
| **Assinatura** | XML-DSig | `assinar_dps_xml_rascunho` → `xml_assinado` (**PFX A1 required** — see [Certificado para assinatura XML](#certificado-para-assinatura-xml)) |
| **Pós-transmissão** | NFSe processada, chave, protocolo | `xml_processado`, `chave_acesso`, `codigo_verificacao`, `protocolo_autorizacao`, `status`; logs em `logs-retorno` |

**Relations loaded at XML build time (expected):** for a complete DPS, the builder should resolve the documento row plus prestador, optional tomador/intermediário, municípios, and all `itens` (with CNA codes)—not only scalar columns on `documentos_fiscais`.

### Agent workflow for DPS changes

1. Locate or add the field in **`layout_dps_leaiute.csv`**; check **`layout_dps.csv`** for RN/reception rules and error codes.
2. Add or update **DB migration** + **`DocumentoForm`** / item forms if persistence is required.
3. Expose in **`frontend/src/resources/definitions.ts`** (and `buildDocumentoBody` / `flattenDocumentoRowToForm` when applicable).
4. Map the field in **`montar_dps_xml`** (or dedicated module per layout version).
5. Add/adjust tests in `backend/app/services/nfse/dps.rs` and documento API tests.
6. Re-run transmission against homologation or mock after changing mandatory nodes.
7. Wire **presence validation** for any new mandatory leiaute field into `validar_dps_contexto` (or a dedicated validator called from assinar/transmitir).
8. Ensure **assinar** rejects missing/invalid certificado per [Certificado para assinatura XML](#certificado-para-assinatura-xml).

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
- **Date format (user preference)**: see [User preferences — date format](#user-preferences--date-format); applies to every date field in the UI.

---

## User preferences — date format

Users choose how **calendar dates** (day/month/year) are shown in inputs and read-only text across the app. This is independent of **theme** and independent of **API/XML** formats (those stay canonical).

### Product rule

- One **per-user** setting controls formatting for **all** date surfaces: resource forms (`FieldDef` `type: 'date'`), read-only show/detail, list columns (e.g. documento competência/emissão), nested panels (itens, certificados, logs), dashboard widgets, and any custom cards that render a date.
- Do **not** format dates ad hoc (`slice(0, 10)`, `toLocaleDateString` per screen, or hard-coded `DD/MM/YYYY`) outside the shared date layer.
- **Date-time** values (e.g. `data_emissao` with time, SEFAZ timestamps, `dhEmi` in DPS) may show an optional time portion; the **date part** still follows the user’s date format. Full ISO/RFC3339 remains the source for API and XML.

### Supported display formats

Store an enum-like code on the user preference; map to display/parsing patterns:

| Preference value | Display example | Notes |
|------------------|-----------------|--------|
| `DMY` | `14/10/2026` | Default for Brazil (dia/mês/ano) |
| `MDY` | `10/14/2026` | US-style |
| `YMD` | `2026-10-14` | ISO-style display (not necessarily the same as API transport) |

Separator may be `/` or `-` as long as it is consistent per pattern; agents should implement via one options table, not scattered strings.

### Configuration UX

- **Where:** hub **Configurações** (`/app/configuracoes`) — section **Preferências** (alongside CNA/municípios links), or equivalent “Minha conta / Preferências” reachable from the authenticated shell.
- **Control:** labeled select (or radio group) “Formato de data” with the options above; short hint that lists and forms will use the chosen format.
- **Persistence:** `users.date_format` (`DMY` | `MDY` | `YMD`) em **`GET /api/me`** (`user.date_format`) e **`PATCH /api/me`**; `DateFormatProvider` sincroniza API + **`localStorage`** por user id.

### Data contract (unchanged)

| Layer | Format |
|-------|--------|
| JSON API request/response | `YYYY-MM-DD` for date fields; date-time fields as ISO 8601 / RFC3339 strings from the backend |
| Form state (`react-hook-form`) | **Canonical string** `YYYY-MM-DD` for `type: 'date'` when submitting via `buildBody` / `buildDocumentoBody` (convert from display format on blur/submit) |
| DPS / SEFIN XML | National layout types (`dCompet`, `dhEmi`, …) — no user formatting |

### Implementation notes for agents

1. **Single module:** e.g. `frontend/src/lib/dateFormat.ts` — `formatDateForDisplay(isoOrYmd, preference)`, `parseDateFromDisplay(text, preference)`, `normalizeApiDateToForm(value)` (existing logic in `resourceForm.ts` for `T`-suffix may move here).
2. **Context:** e.g. `DateFormatProvider` next to `ThemeProvider` / `AuthProvider`, reading preference from `me` + localStorage fallback; expose `useDateFormat()`.
3. **Inputs:** `ResourceFieldInput` for `type: 'date'` — show/edit using the user format (text + mask or locale-aware picker); never rely on raw `<input type="date">` alone if it ignores the product format.
4. **Read-only:** `readOnlyFieldDisplay`, documento list helpers (`documentoListDate`, etc.), show cards, and log tables must call `formatDateForDisplay`, not slice ISO strings.
5. **New screens:** any new date column or field goes through the same helpers; add Vitest cases for DMY/MDY/YMD round-trip and API boundary (`YYYY-MM-DD`).
6. **Backend (when persisting preference):** optional column or JSON on `users` / `companies_usuarios`; include in `MeDto`; validate enum on PATCH. No change to documento or certificado date columns.

**Status:** **implemented** — `frontend/src/lib/dateFormat.ts`, `DateFormatProvider` (`frontend/src/theme/DateFormatContext.tsx`), Preferências em Configurações, `ResourceFieldInput`/`documentoListFormat`/show labels; API `date_format` persistido no usuário.

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

OpenAPI: `docs/api/openapi.yaml` (inclui `assinar`, `xml`, `transmitir`, `PATCH /api/me`); conferir `routes.rs` ao adicionar rotas.

---

## Conventions for agents

1. **Prefer extending `definitions.ts`** over one-off forms when adding CRUD fields or sections.
2. **Documento changes**: update both `buildDocumentoBody` / `flattenDocumentoRowToForm` and backend `DocumentoForm` + normalizers.
3. **CNC**: keep mock-friendly behavior (localhost CNC URL skips mTLS); do not break `applyCncItemToForm` mapping.
4. **Scoped relations**: tomador/intermediário must keep `relationScopeFromField: 'empresa_id'`.
5. **Tests**: frontend Vitest (`frontend/src/**/*.test.ts`, `ResourceCrud.test.tsx`); backend `cargo test`; integration `make test-integration` (MySQL).
6. **Minimal diffs**: match existing patterns (`NestedResourcePanel`, `buildNestedCreateBody`, resource pages).
7. **Unsaved changes**: guard navigation when dirty on all mutating forms; see [Unsaved changes — required behavior](#unsaved-changes--required-behavior). Do not bypass with ad-hoc “leave anyway” flows.
8. **Date format**: all calendar dates in the UI use the user preference via shared helpers; see [User preferences — date format](#user-preferences--date-format). API and XML stay ISO/`YYYY-MM-DD`.
9. **DPS layout**: treat root **`layout_dps_leaiute.csv`** (XML structure) and **`layout_dps.csv`** (RN/reception) as the schema source of truth; align forms, DB, validation, and `montar_dps_xml`—see [DPS XML layout (CSV)](#dps-xml-layout-csv).
10. **DPS mandatory presence**: validate **all** leiaute-required `infDPS` fields for **presence** before assinar/transmitir; extend `validar_dps_contexto` / `dps_layout` as mapping grows—see [Mandatory XML field presence](#mandatory-xml-field-presence).
11. **DPS signing cert**: **Assinar** must use prestador **certificado digital** (A1 PFX) for real XML-DSig; never rely on stub signatures for SEFIN-bound notas—see [Certificado para assinatura XML](#certificado-para-assinatura-xml).

---

## Key file index

| Area | Files |
|------|--------|
| Resource definitions | `frontend/src/resources/definitions.ts` |
| Form core | `frontend/src/lib/resourceForm.ts`, `ResourceFormFields.tsx`, `ResourceFieldInput.tsx` |
| Date format | `frontend/src/lib/dateFormat.ts`, `frontend/src/theme/DateFormatContext.tsx`, `SettingsPage.tsx` → Preferências; `users.date_format`, `session_controller` `update_me` |
| Item tributos UI | `ItemTributosPanel.tsx`, `NestedResourcePanel` + `definitions.ts` `itemTributos` |
| Certificado enum | `frontend/src/lib/certificadoTipo.ts`, `backend/app/services/certificado_tipo.rs` |
| Unsaved + Salvar | `UnsavedConfirmDialog.tsx`, `useRegisterUnsavedSave`, `ResourceEditPage.tsx` |
| Documento | `documentoParteLocalidade.ts`, `documentoItens.ts`, `DocumentoShowView.tsx`, `DocumentoListTable.tsx`, `DocumentoItensDraftPanel.tsx` |
| CNC UI/API | `CncConsultaField.tsx`, `lib/cncConsulta.ts`, `api/cnc.ts`, `ApiCncController` |
| Documento API | `documentos_controller.rs`, `documento_presentacao.rs`, `nfse/dps.rs` |
| DPS layout spec | `layout_dps_leaiute.csv`, `layout_dps.csv` (repo root); `nfse/dps.rs` (XML builder), `nfse/dps_layout.rs`, `nfse/dps_rn.rs`, `nfse/rn_validacao.rs`, `nfse/assinatura.rs` |
| SEFIN/CNC config | `backend/config/development.yml`, `production.yml`, env `SEFIN_BASE_URL`, `CNC_BASE_URL` |
| Routes | `backend/config/routes.rs` |
| Tenant | `backend/app/auth/tenant.rs`, `tenant_scope.rs`, `AuthContext.tsx` |

---

*Last aligned with codebase features: multi-tenant NFSe CRUD, CNC consult, documento list/show/sign/transmit/XML (optional `certificado_id` on transmit when multiple certs), draft itens on create, item tributos nested UI, certificado `tipo` enum (UI + API validation), tributação municipal/federal form sections, cursor pagination, Sonner toasts, unsaved-changes guard with **Salvar** on edit (`UnsavedConfirmDialog`). DPS: canonical **`layout_dps_leaiute.csv`** + **`layout_dps.csv`** at repo root; `montar_dps_xml_context` + `dps_layout`/`dps_rn` (xlsx loaders pending CSV migration) + `validar_dps_contexto`. **Agent policy:** mandatory `infDPS` leiaute fields must be **presence-validated** before assinar/transmitir (`layout_cobertura_parcial`); **assinar** requires prestador **certificado A1** for real XML-DSig. User date format (`DMY`/`MDY`/`YMD`) via API + UI + `todayApiDate()`. Doido **0.0.32-test1**.*

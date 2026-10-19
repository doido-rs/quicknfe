# Sync diário do catálogo NFS-e (E0312)

Carga automática da **lista nacional de CNA**, **regras de incidência** (`MUN.INCID_INFO.SERV.`) e **parametrização municipal** (API ADN — OpenAPI em `{ADN}/parametrizacao/docs`), para validar rejeição **E0312** antes de assinar/transmitir.

## Filas e worker

- Fila doido-jobs: **`catalog`** (configurada em `jobs.queues` nos YAML de ambiente).
- Worker: `cargo doido worker` (processa `default` e `catalog`).

## Jobs

| Job | Função |
|-----|--------|
| `sync_nfse_catalog_daily` | Anexo A (municípios IBGE) → incidência/lista nacional → enfileira coordenador → **reagenda** em +24h (hora `settings.nfse_catalog.daily_hour`, UTC−3). |
| `sync_municipio_cna_coordinator` | Lista todos os `municipios.codigo_ibge`, divide em lotes e enfileira jobs filhos. |
| `sync_municipio_cna_lote` | Por IBGE: para cada CNA em `cnas`, `GET …/historicoaliquotas` e replace em `municipio_cna_parametrizacao`. |

O conector municipal usa **`GET …/historicoaliquotas`**. Para cada CNA nacional tenta `…000`; se a ADN retornar vazio (comum no **Rio — IBGE 3304557**), varre desdobros `001`… (`desdobro_scan_max` / `desdobro_scan_stop_after_misses` em `nfse_catalog`). Ex.: `010101` → `01.01.01.001` (não `.000`).

## Configuração (`settings.adn`)

| Campo / env | Uso |
|-------------|-----|
| `ADN_BASE_URL` | Host ADN (homolog: `https://adn.producaorestrita.nfse.gov.br`). |
| `ADN_MTLS_PFX_BASE64` | Conteúdo do PFX (base64) — **prioridade** sobre arquivo; usado pelo worker e conector ADN. |
| `ADN_MTLS_PFX_PATH` | Caminho do `.pfx` se base64 não estiver definido. |
| `ADN_MTLS_PFX_PASSWORD` | Senha do PFX (`CERTIFICADO_PFX_*` são aliases). |
| `NFSE_ANEXO_A_URL` | Override URL Anexo A (municípios). |
| `NFSE_INCIDENCIA_XLSX_URL` | Planilha com aba `MUN.INCID_INFO.SERV.`; padrão: `layout_dps.xlsx` no repo. |
| `NFSE_ANEXO_B_URL` | Anexo B opcional (lista nacional complementar). |

Em `127.0.0.1` / `localhost`, o ADN **não** usa mTLS (mock). Fora disso, se `ADN_MTLS_PFX_PATH` estiver definido, o worker usa esse PFX nas chamadas de parametrização.

## Bootstrap

`cargo doido db seed` enfileira **uma** execução de `import_nfse_anexo_a` (fila `default`) e de `sync_nfse_catalog_daily` (fila `catalog`) quando `municipio_cna_parametrizacao` estiver **vazia** e não houver job pendente/agendado com o mesmo nome (jobs `dead` não bloqueiam novo bootstrap).

## API QuickNFe (após sync)

- `GET /api/municipios/{id}/cnas?competencia=YYYY-MM-DD` — CNAs **administrados** naquele município.
- `GET /api/cnas` — catálogo nacional completo (referência).

## cURL — reproduzir fontes e ADN (OpenAPI v1)

Variáveis comuns:

```bash
export ADN_BASE="${ADN_BASE_URL:-https://adn.producaorestrita.nfse.gov.br}"
export IBGE=3304557
export CODIGO_SERVICO='01.01.01.000'   # CNA 010101 + desdobro 000
export COMPETENCIA='2026-10-01T00:00:00'
# Opcional mTLS (homolog/produção):
export ADN_MTLS_PFX_PATH=/caminho/emissor-homolog.pfx
export ADN_MTLS_PFX_PASSWORD='sua-senha'
```

Helper para curl com certificado quando o PFX estiver configurado:

```bash
CURL_ADN=(curl -fsS -H 'Accept: application/json')
if [[ -n "${ADN_MTLS_PFX_PATH:-}" ]]; then
  CURL_ADN+=(--cert-type P12 --cert "${ADN_MTLS_PFX_PATH}:${ADN_MTLS_PFX_PASSWORD}")
fi
```

### Catálogo nacional (jobs — não passam pelo ADN)

Anexo A (municípios IBGE + países):

```bash
curl -fsSL -o /tmp/anexo_a.xlsx \
  'https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/documentacao-atual/anexo_a-municipio_ibge-paises_iso2-v1-00-snnfse-20251210.xlsx'
```

Incidência (local ou URL em `NFSE_INCIDENCIA_XLSX_URL`):

```bash
curl -fsSL -o /tmp/incidencia.xlsx "$NFSE_INCIDENCIA_XLSX_URL"
```

### ADN — parametrização municipal (mesmo tráfego do `sync_municipio_cna_lote`)

Documentação OpenAPI embutida:

```bash
"${CURL_ADN[@]}" "${ADN_BASE}/parametrizacao/docs/index.html" -o /tmp/adn-parametrizacao-docs.html
```

**Histórico de alíquotas** (usado pelo QuickNFe para popular `municipio_cna_parametrizacao`):

```bash
"${CURL_ADN[@]}" \
  "${ADN_BASE}/parametrizacao/${IBGE}/${CODIGO_SERVICO}/historicoaliquotas" | jq .
```

**Alíquota na competência** (consulta pontual — OpenAPI `/{competencia}/aliquota`):

```bash
COMP_ENC=$(python3 -c "import urllib.parse; print(urllib.parse.quote('${COMPETENCIA}'))")
"${CURL_ADN[@]}" \
  "${ADN_BASE}/parametrizacao/${IBGE}/${CODIGO_SERVICO}/${COMP_ENC}/aliquota" | jq .
```

**Convênio municipal**:

```bash
"${CURL_ADN[@]}" "${ADN_BASE}/parametrizacao/${IBGE}/convenio" | jq .
```

**Retenções na competência**:

```bash
COMP_ENC=$(python3 -c "import urllib.parse; print(urllib.parse.quote('${COMPETENCIA}'))")
"${CURL_ADN[@]}" \
  "${ADN_BASE}/parametrizacao/${IBGE}/${COMP_ENC}/retencoes" | jq .
```

Resposta típica de alíquotas (`ResultadoConsultaAliquotas`):

```json
{
  "mensagem": null,
  "aliquotas": {
    "01.01.01.000": [{ "Aliq": 2.5, "DtIni": "2024-01-01T00:00:00", "DtFim": null }]
  }
}
```

404 = sem parametrização para aquele par município/serviço (o app grava zero linhas, não aborta o lote).

### Persistir no MySQL (QuickNFe)

```bash
cd backend
cargo doido db migrate
cargo doido db seed    # enfileira import Anexo A + sync diário se parametrização vazia
cargo doido worker     # filas default + catalog
```

Defina `ADN_MTLS_PFX_*` no ambiente do worker se a ADN exigir certificado.

## Tempo estimado

~5,5 mil municípios × ~337 CNAs × 1 chamada `historicoaliquotas` (com `sleep_ms` entre serviços); com `batch_size=50`, planeje **dias** de worker na fila `catalog` na primeira carga.

## Tabelas

- `cnas` — colunas `incid_*`, `grupo_*`, `synced_at`.
- `municipio_cna_parametrizacao` — snapshot por município/código/desdobro.
- `nfse_catalog_sync_runs` — histórico de execuções (`nacional` | `municipios`).

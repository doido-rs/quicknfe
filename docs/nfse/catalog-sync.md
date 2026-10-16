# Sync diário do catálogo NFS-e (E0312)

Carga automática da **lista nacional de CNA**, **regras de incidência** (`MUN.INCID_INFO.SERV.`) e **parametrização municipal** (API ADN), para validar rejeição **E0312** antes de assinar/transmitir.

## Filas e worker

- Fila doido-jobs: **`catalog`** (configurada em `jobs.queues` nos YAML de ambiente).
- Worker: `cargo doido worker` (processa `default` e `catalog`).

## Jobs

| Job | Função |
|-----|--------|
| `sync_nfse_catalog_daily` | Anexo A (municípios IBGE) → incidência/lista nacional → enfileira coordenador → **reagenda** em +24h (hora `settings.nfse_catalog.daily_hour`, UTC−3). |
| `sync_municipio_cna_coordinator` | Lista todos os `municipios.codigo_ibge`, divide em lotes e enfileira jobs filhos. |
| `sync_municipio_cna_lote` | Para cada IBGE do lote: `GET {ADN}/parametros_municipais/{ibge}/aliquotas` e replace em `municipio_cna_parametrizacao`. |

## Variáveis de ambiente

| Variável | Uso |
|----------|-----|
| `ADN_BASE_URL` | Base ADN (homolog: `https://adn.producaorestrita.nfse.gov.br`). |
| `NFSE_ANEXO_A_URL` | Override URL Anexo A (municípios). |
| `NFSE_INCIDENCIA_XLSX_URL` | Planilha com aba `MUN.INCID_INFO.SERV.`; padrão: `layout_dps.xlsx` no repo. |
| `NFSE_ANEXO_B_URL` | Anexo B opcional (lista nacional complementar). |

## Bootstrap

`cargo doido db seed` enfileira **uma** execução de `import_nfse_anexo_a` (fila `default`) e de `sync_nfse_catalog_daily` (fila `catalog`) quando `municipio_cna_parametrizacao` estiver **vazia** e não houver job pendente/agendado com o mesmo nome (jobs `dead` não bloqueiam novo bootstrap).

## API

- `GET /api/municipios/{id}/cnas?competencia=YYYY-MM-DD` — CNAs **administrados** naquele município (após sync).
- `GET /api/cnas` — catálogo nacional completo (referência).

## Tempo estimado

~5,5 mil municípios × 1 chamada ADN/dia; com `batch_size=50` e `sleep_ms=100`, planeje **horas** de worker na fila `catalog` (ajuste `sleep_ms` / concorrência conforme limites ADN).

## Tabelas

- `cnas` — colunas `incid_*`, `grupo_*`, `synced_at`.
- `municipio_cna_parametrizacao` — snapshot por município/código/desdobro.
- `nfse_catalog_sync_runs` — histórico de execuções (`nacional` | `municipios`).

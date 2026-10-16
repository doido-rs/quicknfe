# DPS — campos com validação de presença (QuickNFe)

Antes de **Assinar** / **Transmitir**, `validar_dps_contexto` chama `dps_presenca::validar_presenca_repo`, que cruza o leiaute (`layout_dps_leaiute.csv`) com o mapa em `backend/app/services/nfse/dps_presenca.rs` (`campos_presenca_mapeados`).

**Tier A (owned):** campos obrigatórios do leiaute **fora** de cenários ainda sem dados na UI (`subst`, `comExt`, `atvEvento`, `vDedRed`, `exigSusp`) — ver `mandatory_infdps_fields_owned` em `dps_layout.rs`.

**Tier B:** cenários condicionais; cobrir via slices em [`dps-slice-dod.md`](dps-slice-dod.md) antes de exigir presença.

**Processo:** ao emitir um tag obrigatório Tier A no XML, inclua-o nesta tabela e em `campos_presenca_mapeados` no mesmo PR. Rode `make dps-coverage` após mudanças.

| Campo XML | Fonte no app |
|-----------|----------------|
| `id` | `gerar_id_inf_dps` — `DPS` + IBGE emissão (7) + tp inscr. (1) + CPF/CNPJ (14) + série (5) + nDPS (15) |
| `tpAmb` | `documentos_fiscais.ambiente` |
| `dhEmi` | `data_emissao` |
| `verAplic` | constante no builder |
| `serie` / `nDPS` | `serie`, `numero_documento` (ou `serie_numero_automatico`) |
| `dCompet` | `data_competencia` ou fallback emissão |
| `tpEmit` / `tpEmis` | constante `1` / `tipo_emissao` |
| `cLocEmi` / `cLocPrestacao` | municípios emissão/prestação |
| `CNPJ`/`CPF`, `xNome`, `cMun` | prestador + município prestador |
| `CEP`, `xLgr`, `nro`, `xBairro` | endereço do **prestador** (`empresas.*`); tomador/intermediário Brasil quando `end` é emitido (Phase 3B) |
| `cTribNac`, `xDescServ` | itens da nota |
| `cIntContrib` | `itens.numero_item` (primeira linha) |
| `vServ` | `valor_total_servicos_produtos` |
| `tribISSQN`, `tpRetISSQN`, `regEspTrib` | tributação documento |
| `opSimpNac` | regime prestador |
| `CST`, `tpRetPisCofins` | tributação federal documento |

Campos Tier A obrigatórios no leiaute **sem** entrada nesta tabela ainda não bloqueiam assinatura; expandir o mapa junto com `montar_dps_xml_context`.

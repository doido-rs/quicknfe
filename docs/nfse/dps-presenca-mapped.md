# DPS — campos com validação de presença (QuickNFe)

Antes de **Assinar** / **Transmitir**, `validar_dps_contexto` chama `dps_presenca::validar_presenca_repo`, que cruza o leiaute (`layout_dps_leaiute.csv`) com o mapa em `backend/app/services/nfse/dps_presenca.rs` (`campos_presenca_mapeados`).

**Processo:** ao emitir um tag obrigatório no XML, inclua-o nesta tabela e em `campos_presenca_mapeados` no mesmo PR ([`dps-slice-dod.md`](dps-slice-dod.md)). Rode `make dps-coverage` após mudanças.

| Campo XML | Fonte no app |
|-----------|----------------|
| `tpAmb` | `documentos_fiscais.ambiente` |
| `dhEmi` | `data_emissao` |
| `serie` / `nDPS` | `serie`, `numero_documento` (ou `serie_numero_automatico`) |
| `dCompet` | `data_competencia` ou fallback emissão |
| `cLocEmi` / `cLocPrestacao` | municípios emissão/prestação |
| `CNPJ`/`CPF`, `xNome`, `cMun` | prestador + município prestador |
| `CEP`, `xLgr`, `nro`, `xBairro` | endereço do **prestador** (`empresas.*`) — tomador/intermediário Brasil quando `end` é emitido |
| `cTribNac`, `xDescServ` | itens da nota |
| `cIntContrib` | `itens.numero_item` (primeira linha) |
| `vServ` | `valor_total_servicos_produtos` |
| `tribISSQN`, `opSimpNac`, `regEspTrib` | tributação documento + regime prestador |

Campos obrigatórios no leiaute **sem** entrada nesta tabela ainda não bloqueiam assinatura; expandir o mapa junto com `montar_dps_xml_context`.

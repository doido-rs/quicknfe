# DPS — campos com validação de presença (QuickNFe)

Antes de **Assinar** / **Transmitir**, `validar_dps_contexto` chama `dps_presenca::validar_presenca_repo`, que cruza o leiaute (`layout_dps_leaiute.csv`) com o mapa em `backend/app/services/nfse/dps_presenca.rs` (`campos_presenca_mapeados`).

| Campo XML | Fonte no app |
|-----------|----------------|
| `tpAmb` | `documentos_fiscais.ambiente` |
| `dhEmi` | `data_emissao` |
| `serie` / `nDPS` | `serie`, `numero_documento` (ou `serie_numero_automatico`) |
| `dCompet` | `data_competencia` ou fallback emissão |
| `cLocEmi` / `cLocPrestacao` | municípios emissão/prestação |
| `CNPJ`/`CPF`, `xNome`, `cMun` | prestador + município prestador |
| `cTribNac`, `xDescServ` | itens da nota |
| `vServ` | `valor_total_servicos_produtos` |
| `tribISSQN`, `opSimpNac`, `regEspTrib` | tributação documento + regime prestador |

Campos obrigatórios no leiaute **sem** entrada nesta tabela ainda não bloqueiam assinatura; expandir o mapa junto com `montar_dps_xml_context`.

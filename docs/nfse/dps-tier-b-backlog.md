# DPS Tier B — cenários pendentes (AGENTS gap epic)

Campos obrigatórios do leiaute em caminhos **Tier B** (`dps_layout::TIER_B_PATH_MARKERS` + `IBSCBS`) aguardam slices de produto:

| Slice | Grupo XML | Próximo passo |
|-------|-----------|---------------|
| 3D | `serv/comExt` | Flags/migration exportação + form + `montar_dps_xml` |
| 3E | `infDPS/subst` | Nota substituta (chave + motivo) |
| 3F | `valores/vDedRed` | Documentos de dedução/redução |
| 3G | `serv/atvEvento` | Datas de evento |
| — | `IBSCBS/*` | Reforma tributária (fora do escopo SEFIN-first atual) |

**Concluído neste epic:** exigSusp (`issqn_tp_suspensao`, `issqn_n_processo_suspensao`), totTrib, RN recepção no motor incremental, presença path-aware tomador/intermediário.

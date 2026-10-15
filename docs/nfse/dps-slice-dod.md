# DPS vertical slice — definition of done

Each SEFIN-first slice (addresses, serviço, tributação, …) must satisfy **all** items before merge:

1. **Data** — DB migration + API forms (`definitions.ts`) when new fields are owned by QuickNFe.
2. **XML** — tag in `montar_dps_xml_context` and `coverage::emitted_tags()` in [`backend/app/services/nfse/dps.rs`](../../backend/app/services/nfse/dps.rs).
3. **Presence** — same tag in `campos_presenca_mapeados` + `valor_presente` in [`dps_presenca.rs`](../../backend/app/services/nfse/dps_presenca.rs); update [`dps-presenca-mapped.md`](dps-presenca-mapped.md).
4. **RN** — applicable `layout_dps.csv` Obrig./Rej. rows in [`dps_rn_engine.rs`](../../backend/app/services/nfse/dps_rn_engine.rs).
5. **Tests** — golden XML and/or `validar_dps_contexto` rejection; bump `layout_cobertura_parcial` `min_ratio` when emission grows.
6. **Report** — run `make dps-coverage` (or `scripts/dps_coverage_report.sh`) in CI when touching DPS modules.

**Product rule:** any mandatory tag emitted by the builder must block assinar/transmitir when empty (presence map), not only appear in XML.

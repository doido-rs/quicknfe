# DPS layout CSV files

QuickNFe uses two CSV files at the repository root as the canonical DPS definition (see [AGENTS.md](../../AGENTS.md)).

| File | Source worksheet(s) in `layout_dps.xlsx` |
|------|-------------------------------------------|
| `layout_dps_leaiute.csv` | `LEIAUTE DPS_NFS-e ` |
| `layout_dps.csv` | `RN DPS_NFS-e` + `RN_RECEPCAO_DPS` (column `ORIGEM` identifies the sheet) |

## Regenerating from the national workbook

1. Place or update `layout_dps.xlsx` at the repo root (official SNNFSe package).
2. Export leiaute:

```bash
python3 scripts/export_layout_dps_csv.py
```

Or manually: save **`LEIAUTE DPS_NFS-e `** as UTF-8 CSV → `layout_dps_leaiute.csv`.

3. For RN, export **`RN DPS_NFS-e`** and **`RN_RECEPCAO_DPS`** into `layout_dps.csv` (the script merges them with an `ORIGEM` column).

4. Run backend tests: `cd backend && cargo test layout_dps layout_cobertura`.

Do not commit `layout_dps.xlsx` unless the team explicitly wants it versioned; the CSV pair is what CI and loaders use.

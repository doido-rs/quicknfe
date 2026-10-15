# Layout DPS CSV — refresh process

When the national NFS-e layout changes:

1. Export or update [`layout_dps_leaiute.csv`](../../layout_dps_leaiute.csv) and [`layout_dps.csv`](../../layout_dps.csv) (see [`scripts/export_layout_dps_csv.py`](../../scripts/export_layout_dps_csv.py) and [`docs/nfse/layout-dps-csv.md`](layout-dps-csv.md)).
2. Run `make dps-coverage` and record the new emitted/mandatory ratio.
3. Open vertical slices per [`dps-slice-dod.md`](dps-slice-dod.md) for any new or changed mandatory `infDPS` fields.
4. Update [`AGENTS.md`](../../AGENTS.md) footer baseline (coverage test `min_ratio`, async status) in the same PR as substantive DPS changes.

Do not invent XML tags without a matching leiaute row.

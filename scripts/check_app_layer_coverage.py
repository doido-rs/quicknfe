#!/usr/bin/env python3
"""Enforce AGENTS.md app-layer coverage on backend/app/controllers and app/services.

Reads llvm-cov JSON (--json --summary-only). Every *.rs file under the two trees
must reach --min line coverage (default 85). Pure re-export mod.rs files (≤3
non-blank, non-comment lines) are skipped.

Usage:
  python3 scripts/check_app_layer_coverage.py path/to/coverage.json
  COVERAGE_MIN=85 python3 scripts/check_app_layer_coverage.py coverage.json
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
BACKEND = REPO_ROOT / "backend"
LAYER_DIRS = (
    BACKEND / "app" / "controllers",
    BACKEND / "app" / "services",
)


def is_trivial_mod(path: Path) -> bool:
    if path.name != "mod.rs":
        return False
    text = path.read_text(encoding="utf-8")
    lines = [
        ln.strip()
        for ln in text.splitlines()
        if ln.strip() and not ln.strip().startswith("//")
    ]
    if not lines:
        return True
    return all(re.match(r"^(pub\s+use|pub\s+mod|mod)\s+", ln) for ln in lines)


def source_files() -> list[Path]:
    out: list[Path] = []
    for root in LAYER_DIRS:
        for path in sorted(root.rglob("*.rs")):
            if is_trivial_mod(path):
                continue
            out.append(path)
    return out


def normalize_key(path: str) -> str:
    p = path.replace("\\", "/")
    if "/app/controllers/" in p:
        return "app/controllers/" + p.split("/app/controllers/")[-1]
    if "/app/services/" in p:
        return "app/services/" + p.split("/app/services/")[-1]
    return p


def load_line_coverage(json_path: Path) -> dict[str, tuple[int, int, float]]:
    data = json.loads(json_path.read_text(encoding="utf-8"))
    files = data["data"][0]["files"]
    out: dict[str, tuple[int, int, float]] = {}
    for entry in files:
        key = normalize_key(entry["filename"])
        lines = entry["summary"]["lines"]
        out[key] = (lines["covered"], lines["count"], lines["percent"])
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("json_path", type=Path, help="llvm-cov JSON report")
    parser.add_argument(
        "--min",
        type=float,
        default=float(os.environ.get("COVERAGE_MIN", "85")),
        help="Default minimum line coverage when no floors file (default: COVERAGE_MIN or 85)",
    )
    parser.add_argument(
        "--floors",
        type=Path,
        default=BACKEND / "coverage" / "app_layer_floors.json",
        help="Optional JSON with aggregate_min_percent and per-file floors (ratchet toward 85)",
    )
    args = parser.parse_args()

    if not args.json_path.is_file():
        print(f"error: coverage JSON not found: {args.json_path}", file=sys.stderr)
        return 1

    floors_data: dict | None = None
    if args.floors.is_file():
        floors_data = json.loads(args.floors.read_text(encoding="utf-8"))

    default_min = args.min
    aggregate_min = float(
        floors_data.get("aggregate_min_percent", default_min) if floors_data else default_min
    )
    file_floors: dict[str, float] = (
        {k: float(v) for k, v in floors_data.get("files", {}).items()} if floors_data else {}
    )

    cov = load_line_coverage(args.json_path)
    failures: list[str] = []
    total_lines = 0
    total_covered = 0

    for src in source_files():
        rel = src.relative_to(BACKEND).as_posix()
        key = rel
        covered, count, pct = cov.get(key, (0, 0, 0.0))
        if count == 0:
            line_count = sum(1 for ln in src.read_text(encoding="utf-8").splitlines() if ln.strip())
            if line_count > 0:
                pct = 0.0
                count = line_count
        total_lines += count
        total_covered += covered
        required = file_floors.get(rel, default_min)
        if round(pct, 2) + 1e-9 < round(required, 2):
            failures.append(
                f"  {pct:5.1f}% ({covered}/{count} lines) {rel} — floor {required:.1f}%"
            )

    aggregate = (100.0 * total_covered / total_lines) if total_lines else 0.0
    target = (
        float(floors_data.get("target_per_file_percent", default_min))
        if floors_data
        else default_min
    )
    print(
        f"app/controllers + app/services: {aggregate:.1f}% lines "
        f"({total_covered}/{total_lines}); target {target:.0f}% per file; "
        f"aggregate floor {aggregate_min:.1f}%"
    )

    ok = True
    if round(aggregate, 2) + 1e-9 < round(aggregate_min, 2):
        print(
            f"FAIL aggregate line coverage {aggregate:.1f}% < floor {aggregate_min:.1f}%",
            file=sys.stderr,
        )
        ok = False

    if failures:
        print(f"FAIL {len(failures)} file(s) below {args.min:.0f}%:", file=sys.stderr)
        for line in failures:
            print(line, file=sys.stderr)
        ok = False
    elif ok:
        print("OK app-layer coverage gates satisfied.")

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())

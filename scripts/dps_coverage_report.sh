#!/usr/bin/env bash
# Relatório CI-friendly: cobertura XML × leiaute infDPS + mapa de presença.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/backend"
cargo test -q metrics_print_smoke -- --nocapture 2>&1 | rg -v '^test ' | rg -v '^running ' | rg -v '^$' || true

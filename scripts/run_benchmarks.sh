#!/usr/bin/env bash
# run_benchmarks.sh — runs all benchmarks and updates README.md
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
LUXON_OUT="$REPO_ROOT/luxon_results.txt"

echo "=== 1/3  Running Rust benchmarks ==="
cd "$REPO_ROOT"
cargo bench

echo ""
echo "=== 2/3  Running Luxon benchmarks ==="
node "$SCRIPT_DIR/luxon_bench.js" bench > "$LUXON_OUT"
echo "Luxon results written to: $LUXON_OUT"

echo ""
echo "=== 3/3  Generating README table ==="
PY=$(command -v python3 2>/dev/null || command -v python)
"$PY" "$SCRIPT_DIR/gen_bench_table.py" --luxon "$LUXON_OUT"

echo ""
echo "Done! README.md benchmark table updated."

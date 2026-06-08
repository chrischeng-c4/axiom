#!/usr/bin/env bash
# coverage.sh — Analyze cclab-mamba test coverage
set -euo pipefail

CRATE="cclab-mamba"
CRATE_DIR="crates/$CRATE"
SRC="$CRATE_DIR/src"
TESTS="$CRATE_DIR/tests"
STDLIB="$SRC/runtime/stdlib"

G='\033[0;32m'; Y='\033[0;33m'; R='\033[0;31m'; B='\033[1m'; N='\033[0m'

echo -e "${B}=== cclab-mamba Test Coverage ===${N}"
echo ""

# 1. Collect test list once
LIST=$(cargo test -p "$CRATE" -- --list 2>&1 | grep ': test$')
TOTAL=$(echo "$LIST" | wc -l | tr -d ' ')
echo -e "${B}Total tests:${N} $TOTAL"
echo ""

# 2. Distribution
echo -e "${B}Distribution:${N}"
runtime=$(echo "$LIST" | grep -c '^runtime' || true)
fixtures=$(echo "$LIST" | grep -c 'run_fixture\|run_cpython' || true)
parser=$(echo "$LIST" | grep -c 'test_parse_' || true)
pipeline=$(echo "$LIST" | grep -c 'test_pipeline_' || true)
jit=$(echo "$LIST" | grep -c 'test_jit_\|test_codegen_\|test_llvm_\|test_aot_' || true)
typecheck=$(echo "$LIST" | grep -c 'test_type_\|test_generic_\|test_any_\|test_optional_\|test_protocol_' || true)
ffi=$(echo "$LIST" | grep -c 'ffi\|test_full_ffi\|test_safety_\|test_marshal_\|test_memory_bridge' || true)
async=$(echo "$LIST" | grep -c 'test_tokio_\|test_multi_thread\|test_concurrent_\|test_safepoint_\|test_async_\|test_coroutine_' || true)

printf "  %-25s %4d\n" "Runtime (stdlib+builtins)" "$runtime"
printf "  %-25s %4d\n" "Parse fixtures" "$fixtures"
printf "  %-25s %4d\n" "Parser" "$parser"
printf "  %-25s %4d\n" "Pipeline (AST->HIR->MIR)" "$pipeline"
printf "  %-25s %4d\n" "JIT/Codegen" "$jit"
printf "  %-25s %4d\n" "Type checker" "$typecheck"
printf "  %-25s %4d\n" "FFI" "$ffi"
printf "  %-25s %4d\n" "Threading/Async" "$async"
echo ""

# 3. Stdlib module coverage summary
echo -e "${B}Stdlib Module Coverage:${N}"
total_mods=0; tested_mods=0
thin_mods=(); zero_mods=()

for f in "$STDLIB"/*_mod.rs; do
  mod=$(basename "$f" | sed 's/_mod\.rs//')
  total_mods=$((total_mods + 1))
  count=$(echo "$LIST" | grep -c "^runtime::stdlib::${mod}_mod::tests" || true)
  src_lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$count" -eq 0 ]; then
    zero_mods+=("$mod")
  elif [ "$count" -le 1 ]; then
    thin_mods+=("$mod ($count test, ${src_lines}L)")
    tested_mods=$((tested_mods + 1))
  else
    tested_mods=$((tested_mods + 1))
  fi
done

pct=$((tested_mods * 100 / total_mods))
echo -e "  Modules with tests: ${G}${tested_mods}${N} / ${total_mods} (${pct}%)"

if [ ${#zero_mods[@]} -gt 0 ]; then
  echo -e "  ${R}No tests:${N} ${zero_mods[*]}"
fi
if [ ${#thin_mods[@]} -gt 0 ]; then
  echo -e "  ${Y}Thin coverage (<=1 test):${N}"
  for t in "${thin_mods[@]}"; do echo "    - $t"; done
fi
echo ""

# 4. Per-module detail table
echo -e "${B}Per-Module Detail:${N}"
printf "  %-22s %5s %5s %6s\n" "Module" "Tests" "Lines" "L/T"
printf "  %-22s %5s %5s %6s\n" "------" "-----" "-----" "---"

for f in "$STDLIB"/*_mod.rs; do
  mod=$(basename "$f" | sed 's/_mod\.rs//')
  count=$(echo "$LIST" | grep -c "^runtime::stdlib::${mod}_mod::tests" || true)
  src_lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$count" -gt 0 ]; then
    ratio=$(echo "scale=1; $src_lines / $count" | bc)
  else
    ratio="-"
  fi
  printf "  %-22s %5d %5d %6s\n" "$mod" "$count" "$src_lines" "$ratio"
done
echo ""

# 5. Source vs test line ratio
src_total=$(find "$SRC" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')
test_total=$(find "$TESTS" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')
ratio=$(echo "scale=1; $test_total * 100 / $src_total" | bc)

echo -e "${B}Line Ratio:${N}"
echo "  Source:  ${src_total} lines"
echo "  Tests:   ${test_total} lines"
echo "  Ratio:   ${ratio}% (test/source)"
echo ""

# 6. Source files without inline tests (excluding stdlib)
echo -e "${B}Source files without inline tests:${N}"
untested=0
while IFS= read -r -d '' f; do
  if ! grep -q '#\[cfg(test)\]\|#\[test\]' "$f" 2>/dev/null; then
    echo "  - $(echo "$f" | sed "s|$SRC/||")"
    untested=$((untested + 1))
  fi
done < <(find "$SRC" -name '*.rs' ! -path '*/stdlib/*' -print0)
echo "  Total: $untested files"

# 7. Line coverage via tarpaulin (if JSON report exists)
TARP_JSON="/tmp/tarp/tarpaulin-report.json"
if [ -f "$TARP_JSON" ]; then
  echo ""
  echo -e "${B}Line Coverage (tarpaulin):${N}"

  # Parse tarpaulin JSON per subsystem using python3
  python3 - "$TARP_JSON" "$SRC" <<'PYEOF'
import json, sys, os

report_path = sys.argv[1]
src_prefix = sys.argv[2]

with open(report_path) as f:
    data = json.load(f)

# Tarpaulin JSON: list of file objects with "path", "content", "traces"
# Each trace has "line" and "stats" with "Line" count (0 = not covered, >0 = covered)
subsystems = {
    "Runtime core": "runtime/",
    "Stdlib": "runtime/stdlib/",
    "Parser": "parser/",
    "Lexer": "lexer/",
    "Type checker": "typechecker/",
    "HIR/MIR/Lowering": ["hir/", "mir/", "lowering/"],
    "Codegen (JIT/AOT)": ["codegen/", "jit/", "aot/"],
    "FFI": "ffi/",
    "Name resolution": "resolver/",
}

# Collect per-file stats
file_stats = {}
for file_obj in data:
    path = file_obj.get("path", "")
    if src_prefix not in path:
        continue
    rel = path.split(src_prefix + "/")[-1] if src_prefix + "/" in path else path
    traces = file_obj.get("traces", [])
    coverable = len(traces)
    covered = sum(1 for t in traces if t.get("stats", {}).get("Line", 0) > 0)
    file_stats[rel] = (covered, coverable)

# Aggregate per subsystem
def match_subsystem(rel, patterns):
    if isinstance(patterns, str):
        patterns = [patterns]
    return any(rel.startswith(p) for p in patterns)

total_covered = 0
total_coverable = 0

print(f"  {'Subsystem':<25} {'Covered':>8} {'Coverable':>10} {'Coverage':>9}")
print(f"  {'-'*25} {'-'*8} {'-'*10} {'-'*9}")

# Process stdlib separately from runtime core
for name, patterns in subsystems.items():
    covered = 0
    coverable = 0
    for rel, (c, t) in file_stats.items():
        if name == "Runtime core":
            # runtime/ but NOT runtime/stdlib/
            if rel.startswith("runtime/") and not rel.startswith("runtime/stdlib/"):
                covered += c
                coverable += t
        elif match_subsystem(rel, patterns):
            covered += c
            coverable += t
    if coverable > 0:
        pct = covered * 100.0 / coverable
        print(f"  {name:<25} {covered:>8} {coverable:>10} {pct:>8.1f}%")
        total_covered += covered
        total_coverable += coverable

if total_coverable > 0:
    total_pct = total_covered * 100.0 / total_coverable
    print(f"  {'─'*25} {'─'*8} {'─'*10} {'─'*9}")
    print(f"  {'TOTAL':<25} {total_covered:>8} {total_coverable:>10} {total_pct:>8.1f}%")
PYEOF

else
  echo ""
  echo -e "${Y}Line coverage (tarpaulin): no report found.${N}"
  echo "  Run: cargo tarpaulin -p cclab-mamba --skip-clean --out Json --output-dir /tmp/tarp"
fi

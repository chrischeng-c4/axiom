#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../.." && pwd -P)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-candidate-root.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

outer_root="$temporary_root/outer"
nested_root="$outer_root/candidate"
mkdir -p "$nested_root/apps/sift"
git -c core.fsmonitor=false init -q "$outer_root"
outer_root="$(cd "$outer_root" && pwd -P)"
nested_root="$(cd "$nested_root" && pwd -P)"
cp "$repo_root/apps/sift/test.sh" "$nested_root/apps/sift/test.sh"

reported_root="$(env -u SIFT_REPO_ROOT \
  bash "$nested_root/apps/sift/test.sh" --print-repo-root)"
[[ "$reported_root" == "$nested_root" ]] || {
  echo "candidate test entrypoint selected an enclosing Git checkout" >&2
  exit 1
}

if SIFT_REPO_ROOT="$outer_root" \
    bash "$nested_root/apps/sift/test.sh" --print-repo-root \
    >"$temporary_root/mismatch.stdout" 2>"$temporary_root/mismatch.stderr"; then
  echo "candidate test entrypoint accepted a mismatched SIFT_REPO_ROOT" >&2
  exit 1
fi
grep -F \
  "SIFT_REPO_ROOT must match the repository that contains apps/sift/test.sh" \
  "$temporary_root/mismatch.stderr" >/dev/null

SIFT_REPO_ROOT="$nested_root" \
  bash "$nested_root/apps/sift/test.sh" --print-repo-root \
  | grep -Fx "$nested_root" >/dev/null

fake_sift="$temporary_root/old-sift"
printf '#!/usr/bin/env bash\nexit 0\n' > "$fake_sift"
chmod +x "$fake_sift"
if SIFT_REPO_ROOT="$nested_root" \
    SIFT_BIN="$fake_sift" \
    SIFT_SOURCE_REVISION="0123456789abcdef0123456789abcdef01234567" \
    bash "$nested_root/apps/sift/test.sh" --candidate \
    >"$temporary_root/sift-bin.stdout" 2>"$temporary_root/sift-bin.stderr"; then
  echo "candidate test entrypoint accepted a caller-supplied SIFT_BIN" >&2
  exit 1
fi
grep -F "Sift candidate mode does not accept a caller-supplied SIFT_BIN" \
  "$temporary_root/sift-bin.stderr" >/dev/null

# The compliance entrypoint must reject a stale fixed-path binary before it
# downloads the upstream suite or starts Docker.
fixed_target="$nested_root/target"
mkdir -p "$fixed_target/debug" "$temporary_root/fake-tools"
cat > "$fixed_target/debug/sift" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "acceptance-build-info" ]]; then
  printf '%s\n' '{"git_sha":"ffffffffffffffffffffffffffffffffffffffff"}'
  exit 0
fi
exit 97
EOF
chmod +x "$fixed_target/debug/sift"
for command in cargo docker tar; do
  cat > "$temporary_root/fake-tools/$command" <<'EOF'
#!/usr/bin/env bash
exit 98
EOF
  chmod +x "$temporary_root/fake-tools/$command"
done
cat > "$temporary_root/fake-tools/curl" <<EOF
#!/usr/bin/env bash
printf '%s\n' called > "$temporary_root/network-called"
exit 99
EOF
chmod +x "$temporary_root/fake-tools/curl"
if PATH="$temporary_root/fake-tools:$PATH" \
    SIFT_REPO_ROOT="$nested_root" \
    CARGO_TARGET_DIR="$fixed_target" \
    SIFT_EXPECTED_SOURCE_REVISION="0123456789abcdef0123456789abcdef01234567" \
    bash "$repo_root/apps/sift/e2e/prometheus_compliance.sh" \
    >"$temporary_root/revision.stdout" 2>"$temporary_root/revision.stderr"; then
  echo "Prometheus compliance accepted a stale fixed-path Sift binary" >&2
  exit 1
fi
grep -F \
  "candidate binary Git SHA mismatch: expected 0123456789abcdef0123456789abcdef01234567" \
  "$temporary_root/revision.stderr" >/dev/null
if [[ -e "$temporary_root/network-called" ]]; then
  echo "Prometheus compliance used the network before it rejected the stale binary" >&2
  exit 1
fi

echo "candidate root isolation: ok"

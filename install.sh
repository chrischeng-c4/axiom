#!/usr/bin/env bash
# cclab installer — auto-dispatches to projects/<name>/install.sh.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/cclab/main/install.sh | bash
#                                                       # default: cap (prebuilt binary)
#   curl -fsSL .../install.sh | bash -s -- --project=score
#                                                       # alt: build score from source
#   bash install.sh --list                              # discover (local checkout)
#
# Private repo:
#   The repo is currently private. Both this script and the cap installer
#   need a token to fetch raw files + release assets. Either:
#     export GH_TOKEN=$(gh auth token)
#   then prefix BOTH curl and bash with the token so it survives the pipe:
#     curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
#       https://.../main/install.sh | GH_TOKEN="${GH_TOKEN}" bash
#   (the curl `-H` is only needed while the repo is private; `gh` CLI auth
#    via `gh auth login` is also accepted when present.)
#
# Resolution order for `--project=<name>`:
#   1. local file  projects/<name>/install.sh   (running inside a checkout)
#   2. remote      https://raw.githubusercontent.com/<repo>/main/projects/<name>/install.sh
#   3. (legacy)    inline score build  — kept until projects/agentic-workflow/install.sh lands
#
# Env:
#   CCLAB_REPO       GitHub repo for remote fetch (default: chrischeng-c4/cclab)
#   CCLAB_REF        git ref/branch for remote fetch (default: main)
#   GH_TOKEN         GitHub token for private-repo fetch (also: GITHUB_TOKEN)
#
# Per-project env vars (e.g. CAP_VERSION, CAP_INSTALL, CAP_REPO) are inherited.
set -euo pipefail

REPO="${CCLAB_REPO:-chrischeng-c4/cclab}"
REF="${CCLAB_REF:-main}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

# If no env token but `gh` is logged in, grab one transparently.
if [ -z "${TOKEN}" ] && command -v gh >/dev/null 2>&1; then
    if gh auth status >/dev/null 2>&1; then
        TOKEN="$(gh auth token 2>/dev/null || true)"
    fi
fi

PROJECT=""
LIST=false
for arg in "$@"; do
    case "${arg}" in
        --project=*) PROJECT="${arg#--project=}" ;;
        --list)      LIST=true ;;
        -h|--help)
            sed -n '2,30p' "$0" 2>/dev/null || cat <<'EOF'
Usage: install.sh --project=<name> | --list
EOF
            exit 0
            ;;
        *) echo "install.sh: unknown arg: ${arg}" >&2; exit 2 ;;
    esac
done

# --list: best-effort enumeration. Only meaningful from a checkout.
if [ "${LIST}" = true ]; then
    if [ -d projects ]; then
        echo "available projects (local checkout):"
        for f in projects/*/install.sh; do
            [ -f "${f}" ] || continue
            name="${f#projects/}"; name="${name%/install.sh}"
            echo "  ${name}"
        done
    else
        echo "no local checkout. browse: https://github.com/${REPO}/tree/${REF}/projects"
    fi
    exit 0
fi

# Default = cap. Prebuilt binary, no toolchain needed — the common case.
# Use --project=score for the legacy local-build path.
PROJECT="${PROJECT:-cap}"

echo "=== ${PROJECT} install ==="

# --- 1. local installer ----------------------------------------------------
local_path="projects/${PROJECT}/install.sh"
if [ -f "${local_path}" ]; then
    # Forward the token through env — child installer needs it too for
    # any GitHub API / asset fetches on a private repo.
    GH_TOKEN="${TOKEN}" exec sh "${local_path}"
fi

# --- 2. remote installer ---------------------------------------------------
remote_url="https://raw.githubusercontent.com/${REPO}/${REF}/projects/${PROJECT}/install.sh"
tmp="$(mktemp 2>/dev/null || mktemp -t cclab-install)"
trap 'rm -f "${tmp}"' EXIT INT TERM

# Build curl auth flag. `-H` with an empty header is rejected by some
# curl builds, so only add it when we actually have a token.
# (avoid bash arrays — macOS bash 3.2 chokes on empty-array expansion
# under `set -u`.)
if [ -n "${TOKEN}" ]; then
    http_code="$(
        curl -fsSL -o "${tmp}" -w '%{http_code}' \
            -H "Authorization: Bearer ${TOKEN}" \
            "${remote_url}" 2>/dev/null || true
    )"
else
    http_code="$(
        curl -fsSL -o "${tmp}" -w '%{http_code}' \
            "${remote_url}" 2>/dev/null || true
    )"
fi

if [ -s "${tmp}" ] && [ "${http_code}" = "200" ]; then
    GH_TOKEN="${TOKEN}" exec sh "${tmp}"
fi

# Useful diagnostic when private repo + no token.
if [ "${http_code}" = "404" ] && [ -z "${TOKEN}" ]; then
    echo "error: 404 on ${remote_url}" >&2
    echo "       repo may be private. Set GH_TOKEN (or run \`gh auth login\`) and retry:" >&2
    echo "         export GH_TOKEN=\$(gh auth token)" >&2
    echo "         curl -fsSL -H \"Authorization: Bearer \$GH_TOKEN\" ${remote_url%/*}/install.sh | GH_TOKEN=\$GH_TOKEN bash" >&2
    exit 1
fi

# --- 3. legacy inline fallback (score only, until it gets its own installer)
if [ "${PROJECT}" = "score" ]; then
    echo ""
    if ! git rev-parse --show-toplevel >/dev/null 2>&1; then
        echo "error: score install needs a cloned cclab checkout (no prebuilt release yet)." >&2
        exit 1
    fi
    cd "$(git rev-parse --show-toplevel)"

    if ! command -v rustup >/dev/null 2>&1; then
        echo "Installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
    echo "rustup: $(rustup --version 2>&1 | head -1)"
    echo "cargo:  $(cargo --version)"
    echo ""

    echo "Building score..."
    cargo build -p score

    rm -f ~/.cargo/bin/score
    cp target/debug/score ~/.cargo/bin/score
    chmod +x ~/.cargo/bin/score
    # Re-sign — macOS arm64 SIGKILLs unsigned cp'd binaries (137).
    codesign -s - -f ~/.cargo/bin/score 2>/dev/null || true
    echo "Installed: $(~/.cargo/bin/score --version 2>&1 || echo 'score')"
    echo ""
    echo "score installed successfully."
    exit 0
fi

# --- 4. unknown project ----------------------------------------------------
echo "no installer found for project '${PROJECT}'" >&2
echo "  tried local:  ${local_path}" >&2
echo "  tried remote: ${remote_url}  (http ${http_code:-?})" >&2
echo "browse available: https://github.com/${REPO}/tree/${REF}/projects" >&2
exit 2

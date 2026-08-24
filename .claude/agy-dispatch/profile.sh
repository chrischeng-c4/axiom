# agy-dispatch project profile — mamba
# Contract: ~/.claude/skills/agy-dispatch/profiles/README.md
# Sourced by agy-wave.sh. No output, no side effects.

REPO="chrischeng-c4/axiom"

# cargo rebuilds the pinned test binary. `lock` denies it globally; without that
# the global grants file hands out `command(cargo test)` on its own.
BUILD_CMDS=(cargo)

# The `-cfbeef43138f9938` suffix is cargo's METADATA hash, not a content hash:
# it is stable across source edits, so a rebuild overwrites the artifact in
# place under the same name. A binary inside target/ is therefore never pinned —
# only the sha256 distinguishes the wave's binary from its successor. This path
# is the staged copy OUTSIDE target/; point AGY_BIN back into target/ only to
# re-pin a wave deliberately. Fired for real on 2026-07-27: a concurrent
# mamba-dev build replaced the target/ artifact mid-wave and the sha guard
# caught it.
BIN="/tmp/mamba-agy/pinned/cpython_ported_integration-d68ad9f6081977f"
SHA="d68ad9f6081977f56698435a1e978b7c9418a06456d47a46d4bcb96f9312cc72"

WITNESS='For a ported CPython fixture the witness is the executed Python:
everything below the closing `# ///` of the PEP 723 header, and — in the
fixtures that carry a `# --- test body ---` marker — everything below that
marker. The header is metadata, the text above the marker is imported CPython
scaffolding the fixture never runs, and a token found only there decides
nothing.'

# Narrow on purpose: target/ churns constantly and would swamp the snapshot.
TREE_WATCH="projects/mamba/src projects/mamba/tests projects/mamba/CAPABILITIES.md"

EXTRA_ADD_DIRS="/tmp/waveA"

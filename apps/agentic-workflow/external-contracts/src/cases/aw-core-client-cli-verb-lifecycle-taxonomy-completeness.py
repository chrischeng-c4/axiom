"""Black-box contract for CLI verb lifecycle taxonomy completeness (#3306).

Independently reconstructs the real, compiled binary's leaf clap-verb tree by
recursively parsing `aw <path...> --help` (never re-running the named cargo
test as oracle) and cross-checks it against `VERB_LIFECYCLE_REGISTRY`,
regex-extracted straight from the live `chain.rs` source, proving every
currently dispatchable leaf verb carries an explicit lifecycle class and
mutation classification with no dangling registry entries.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import REPOSITORY_ROOT, run_aw

CASE_ID = "aw-core-client-cli-verb-lifecycle-taxonomy-completeness"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "cli-verb-lifecycle-taxonomy-completeness"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-cli-verb-lifecycle-taxonomy-completeness"
)
ASSERTIONS = (
    "recursively parsing --help output from the real compiled binary (never "
    "re-running the named cargo test as oracle), skipping clap's own help "
    "pseudo-subcommand, reconstructs a leaf-verb-path set that is exactly "
    "equal -- no more, no fewer -- to the path fields regex-extracted from "
    "the live VERB_LIFECYCLE_REGISTRY array in chain.rs, proving every "
    "currently dispatchable verb carries an explicit lifecycle "
    "classification and the registry carries no dangling entry for a "
    "removed verb",
    "every regex-extracted registry entry's Migration-vs-non-Migration "
    "class correlates exactly with a non-empty-vs-empty sunset_criterion "
    "field -- every Migration entry declares a concrete retirement "
    "condition and every Core/Utility entry declares none -- proving the "
    "tracked-state mutation/removal classification is a real two-field "
    "invariant enforced on every entry rather than present on only some",
)

_CHAIN_RS = REPOSITORY_ROOT / "apps/agentic-workflow/src/cli/chain.rs"

_REGISTRY_START = "const VERB_LIFECYCLE_REGISTRY: &[VerbLifecycle] = &["
_ENTRY_RE = re.compile(r"VerbLifecycle \{(.*?)\n    \},", re.DOTALL)
_PATH_RE = re.compile(r'path:\s*"([^"]*)"')
_CLASS_RE = re.compile(r"class:\s*VerbLifecycleClass::(\w+)")


def _extract_registry() -> list[dict[str, object]]:
    text = _CHAIN_RS.read_text(encoding="utf-8")
    start = text.index(_REGISTRY_START) + len(_REGISTRY_START)
    end = text.index("\n];", start)
    body = text[start:end]

    entries: list[dict[str, object]] = []
    for match in _ENTRY_RE.finditer(body):
        inner = match.group(1)
        path_match = _PATH_RE.search(inner)
        class_match = _CLASS_RE.search(inner)
        assert path_match is not None, inner
        assert class_match is not None, inner
        entries.append(
            {
                "path": path_match.group(1),
                "class": class_match.group(1),
                "sunset_empty": 'sunset_criterion: "",' in inner,
            }
        )
    assert len(entries) >= 90, f"suspiciously few registry entries parsed: {len(entries)}"
    return entries


def _subcommand_names(help_text: str) -> list[str]:
    lines = help_text.splitlines()
    names: list[str] = []
    in_commands = False
    for line in lines:
        if line.strip() == "Commands:":
            in_commands = True
            continue
        if not in_commands:
            continue
        if not line.strip():
            break
        match = re.match(r"^  (\S+)", line)
        if not match:
            break
        names.append(match.group(1))
    return names


def _leaf_verb_paths(root: Path) -> set[str]:
    leaves: set[str] = set()

    def walk(parts: list[str]) -> None:
        help_text = run_aw(root, *parts, "--help").stdout
        subcommands = [name for name in _subcommand_names(help_text) if name != "help"]
        if not subcommands:
            if parts:
                leaves.add(".".join(parts))
            return
        for name in subcommands:
            walk(parts + [name])

    walk([])
    return leaves


def verify() -> list[str]:
    registry = _extract_registry()
    registry_paths = {entry["path"] for entry in registry}
    assert len(registry_paths) == len(registry), "duplicate path in VERB_LIFECYCLE_REGISTRY"

    live_leaves = _leaf_verb_paths(REPOSITORY_ROOT)
    assert len(live_leaves) >= 90, f"suspiciously few live leaf verbs discovered: {live_leaves}"

    missing_from_registry = live_leaves - registry_paths
    dangling_in_registry = registry_paths - live_leaves
    assert not missing_from_registry, f"unclassified live verb(s): {sorted(missing_from_registry)}"
    assert not dangling_in_registry, f"dangling registry entry(ies): {sorted(dangling_in_registry)}"
    assert live_leaves == registry_paths, (live_leaves, registry_paths)

    for entry in registry:
        is_migration = entry["class"] == "Migration"
        sunset_empty = entry["sunset_empty"]
        if is_migration:
            assert not sunset_empty, f"Migration entry with empty sunset_criterion: {entry['path']}"
        else:
            assert sunset_empty, f"non-Migration entry with a sunset_criterion: {entry['path']}"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()

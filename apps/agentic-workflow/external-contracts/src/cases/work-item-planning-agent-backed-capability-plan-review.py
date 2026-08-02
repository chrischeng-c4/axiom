"""Black-box contract for collision-safe capability-plan publication (#3328).

`aw capability sweep --write-wi-plans` stages capability-plan candidates for
review, and `aw wi plan-review` accepts or rejects them. For the
`capability_plan` kind, an accepted decision publishes every candidate as a
work item in the same call. Two candidates whose titles collide on the
local-backend slug budget must both persist as distinct work items, and
`published_issue_count` must equal the number of work items actually written
to disk rather than the raw candidate count regardless of collisions.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from wi_contract_fixture import (
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "work-item-planning-agent-backed-capability-plan-review"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "agent-backed-capability-plan-review"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-agent-backed-capability-plan-review"
)
ASSERTIONS = (
    "capability-plan review rejects an agent reviewer whose identity matches the recorded plan author",
    "an independent agent reviewer with a fully satisfied checklist publishes the pending capability-plan candidates",
    "two capability-plan candidates whose titles collide on the local-backend slug budget both persist as distinct work items",
    "published_issue_count in the accepted review response equals the number of distinct work items actually written to disk",
)

DEMO_READINESS_COMMAND = (
    "uv run --frozen --offline --project . python src/runner.py --case demo-readiness"
)

CANDIDATE_TITLES = [
    "Close capability claim: Alpha Service / first-candidate-row",
    "Close capability claim: Alpha Service / second-candidate-row",
]


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated capability-plan collision fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Alpha Service | - | planned | none | smoke | not-ready | Capability-plan collision fixture |

### Alpha Service

ID: alpha-service
Type: DeveloperTool
Surfaces: CLI: `true` - fixture surface.
EC Dimensions: behavior: `true` - fixture dimension.
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Prove that colliding claim slugs both publish as distinct work items.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| First candidate row | change | - | planned | none | smoke | `true` |
| Second candidate row | change | - | planned | none | smoke | `true` |
"""


def _write_fixture(root: Path) -> None:
    """Stand up a project whose Python-artifact readiness is fully ready.

    `aw capability sweep` only routes an open capability gap to `create_wi`
    (and only then does `--write-wi-plans` stage a capability-plan review)
    once the project's Python TD/EC readiness projection reports no
    blockers -- otherwise the sweep's next action is forced to
    `aw ec check`/`aw ec verify` regardless of any open capability gap. The
    skeleton below is the minimal ready TD module plus EC case proven by
    `capability-control-plane-python-artifact-readiness.py`; only the
    capability document differs, carrying the two colliding claim rows this
    case actually exercises.
    """
    project = root / "project"
    td_root = project / "tech-design"
    ec_root = project / "external-contracts"
    (root / ".git").mkdir(exist_ok=True)
    (td_root / "src/demo/public_contracts").mkdir(parents=True, exist_ok=True)
    (ec_root / "src/cases").mkdir(parents=True, exist_ok=True)
    (ec_root / "evidence").mkdir(exist_ok=True)
    (root / "aw.toml").write_text(
        """version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "project"
td_path = "project/tech-design"
cap_path = "project/CAPABILITIES.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["project/**"]
target = "python"
test_cmd = "true"
""",
        encoding="utf-8",
    )
    (project / "CAPABILITIES.md").write_text(_capability_document(), encoding="utf-8")
    (td_root / "pyproject.toml").write_text(
        """[project]
name = "demo-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    (td_root / "src/demo/public_contracts/readiness.py").write_text(
        '''__aw_artifact_id__ = "artifact:demo/readiness"
__aw_public_contract__ = True


def demo_readiness() -> str:
    return "Python artifact readiness"
''',
        encoding="utf-8",
    )
    (ec_root / "src/runner.py").write_text(
        '''from __future__ import annotations

import hashlib
import json
import os
import runpy
import sys
from pathlib import Path


CASE_ID = "demo-readiness"
DECLARED_COMMAND = (
    "uv run --frozen --offline --project . python "
    "src/runner.py --case demo-readiness"
)


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def main() -> int:
    if sys.argv[1:] != ["--case", CASE_ID]:
        raise RuntimeError(f"expected --case {CASE_ID}")
    root = Path(__file__).resolve().parents[1]
    implementation = root / "src/cases/readiness.py"
    verifier = runpy.run_path(str(implementation))["verify"]
    assertions = verifier()
    if not assertions:
        raise RuntimeError("fixture verifier executed zero assertions")
    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": CASE_ID,
        "mode": "behavior",
        "source_digest": os.environ["AW_PYTHON_EC_SOURCE_DIGEST"],
        "declared_command": DECLARED_COMMAND,
        "implementation": "src/cases/readiness.py",
        "implementation_digest": digest_bytes(implementation.read_bytes()),
        "exit_code": 0,
        "assertions": assertions,
        "attempts": [
            {
                "exit_code": 0,
                "assertion_count": len(assertions),
            }
        ],
    }
    evidence_path = root / "evidence/readiness.json"
    evidence_path.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\\n",
        encoding="utf-8",
    )
    print(json.dumps(evidence, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
''',
        encoding="utf-8",
    )
    (ec_root / "src/cases/readiness.py").write_text(
        'def verify() -> list[str]:\n'
        '    return ["readiness is externally observable"]\n',
        encoding="utf-8",
    )
    (ec_root / "pyproject.toml").write_text(
        """[project]
name = "demo-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "fixture:external"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "demo-readiness"
artifact_id = "artifact:demo/readiness"
capability_id = "demo-capability"
use_case_id = "demo-readiness"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/readiness.py"
promise = "the readiness projection reports this case's evidence state"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "uv run --frozen --offline --project . python src/runner.py --case demo-readiness"
evidence_paths = ["evidence/readiness.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "readiness")
    _execute_readiness_verifier(ec_root)


def _independent_source_digest(ec_root: Path) -> str:
    ignored = {
        "__pycache__",
        ".venv",
        "venv",
        ".pytest_cache",
        ".mypy_cache",
        ".ruff_cache",
        ".tox",
        "build",
        "dist",
        ".eggs",
    }
    files = sorted(
        path
        for path in (ec_root / "src").rglob("*.py")
        if not any(part in ignored for part in path.relative_to(ec_root).parts)
    )
    assert files, ec_root
    digest = hashlib.sha256()
    for path in files:
        relative = path.relative_to(ec_root).as_posix().encode()
        body = path.read_bytes()
        digest.update(relative)
        digest.update(b"\0")
        digest.update(len(body).to_bytes(8, byteorder="big"))
        digest.update(b"\0")
        digest.update(body)
    return "sha256:" + digest.hexdigest()


def _execute_readiness_verifier(ec_root: Path) -> None:
    source_digest = _independent_source_digest(ec_root)
    env = os.environ.copy()
    env["AW_PYTHON_EC_SOURCE_DIGEST"] = source_digest
    completed = subprocess.run(
        [
            "uv",
            "run",
            "--frozen",
            "--offline",
            "--project",
            ".",
            "python",
            "src/runner.py",
            "--case",
            "demo-readiness",
        ],
        cwd=ec_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            "demo-readiness fixture verifier failed:\n"
            f"stdout={completed.stdout}\nstderr={completed.stderr}"
        )


def _workspace_slug(project_root: Path) -> str:
    """Mirror `workspace_cache_slug` in `src/shared/workspace.rs`.

    The capability-plan review/manifest payload the sweep writes lives under
    the ephemeral `/tmp/aw/workspaces/<slug>` runtime root rather than inside
    the fixture's own tempdir, and the slug is this exact deterministic
    projection of the canonicalized project root -- independently
    recomputing it (rather than trusting any single glob) keeps this case
    scoped to its own tempdir even under concurrent EC runs against other
    fixtures also named `demo`.
    """
    raw = str(project_root.resolve())
    out: list[str] = []
    last_dash = True
    for char in raw:
        if char.isascii() and char.isalnum():
            out.append(char.lower())
            last_dash = False
        elif not last_dash:
            out.append("-")
            last_dash = True
    trimmed = "".join(out).strip("-")
    return trimmed or "workspace"


def _pending_review_record(root: Path) -> dict[str, Any]:
    slug = _workspace_slug(root)
    payload_dir = (
        Path("/tmp/aw/workspaces") / slug / "payloads" / "capability-plan" / "demo"
    )
    found = sorted(payload_dir.glob("*/review.json"))
    assert len(found) == 1, (payload_dir, found)
    return json.loads(found[0].read_text(encoding="utf-8"))


def _author_identity(pending: dict[str, Any]) -> str:
    plan_path = Path(str(pending["plan_path"]))
    author_path = plan_path.with_suffix(".author.json")
    author = json.loads(author_path.read_text(encoding="utf-8"))
    return str(author["author"])


def _last_json_value(completed: subprocess.CompletedProcess[str]) -> Any:
    raw = completed.stdout
    decoder = json.JSONDecoder()
    values: list[Any] = []
    cursor = 0
    while cursor < len(raw):
        while cursor < len(raw) and raw[cursor].isspace():
            cursor += 1
        if cursor >= len(raw):
            break
        value, cursor = decoder.raw_decode(raw, cursor)
        values.append(value)
    if not values:
        raise AssertionError(f"command emitted no JSON:\nstderr={completed.stderr}")
    return values[-1]


def _accepted_record(
    pending: dict[str, Any], *, reviewed_by: str
) -> dict[str, Any]:
    record = dict(pending)
    record.update(
        decision="accepted",
        reviewer_kind="agent",
        reviewed_by=reviewed_by,
        reviewed_at="2026-01-01T00:00:00Z",
        summary="independent review of the capability-plan collision fixture",
        checklist={
            "capability_claim_coverage": True,
            "scope_coverage": True,
            "bounded_candidates": True,
            "tracker_reconciliation": True,
            "verification_specific": True,
            "priority_consistent": True,
            "no_duplicate_wis": True,
            "publication_safe": True,
        },
        findings=[],
    )
    return record


def _issues_root(root: Path) -> Path:
    """Local backend on-disk root: `/tmp/aw/workspaces/<slug>/issues`.

    `Issue.slug` carries `#[serde(skip)]` (it lives in the filename, not the
    frontmatter), so `aw wi list --json` never emits a `slug` field. The
    definitive proof that two colliding candidate titles produced two
    distinct work items -- rather than the second `create()` silently
    overwriting the first -- is two distinct files on disk, so this reads
    the local backend's real issue directory instead of asking the JSON
    projection for something it deliberately never carries.
    """
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues"


def _issue_stems_for_titles(root: Path, titles: list[str]) -> dict[str, str]:
    """Map each `title` in `titles` to the on-disk filename stem (slug) of
    the single issue file whose frontmatter/body carries that exact title.
    """
    stems_by_title: dict[str, str] = {}
    for subdir in ("open", "closed"):
        directory = _issues_root(root) / subdir
        if not directory.is_dir():
            continue
        for md_path in sorted(directory.glob("*.md")):
            text = md_path.read_text(encoding="utf-8")
            for title in titles:
                if title in text:
                    assert title not in stems_by_title, (
                        title,
                        stems_by_title[title],
                        md_path,
                    )
                    stems_by_title[title] = md_path.stem
    return stems_by_title


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-capability-plan-collision-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root)

        sweep = _last_json_value(
            run_aw(
                root,
                "capability",
                "sweep",
                "--include-issue-inventory",
                "--write-wi-plans",
            )
        )
        project_entry = next(
            entry for entry in sweep["projects"] if entry["project"] == "demo"
        )
        assert project_entry["next_action_kind"] == "create_wi", project_entry

        pending = _pending_review_record(root)
        assert pending["decision"] == "pending", pending
        assert pending["kind"] == "capability_plan", pending

        manifest = json.loads(
            Path(str(pending["manifest_path"])).read_text(encoding="utf-8")
        )
        candidates = manifest["candidates"]
        assert len(candidates) == 2, candidates
        titles = sorted(candidate["title"] for candidate in candidates)
        assert titles == CANDIDATE_TITLES, titles

        author_identity = _author_identity(pending)

        not_independent_path = root / "review-not-independent.json"
        not_independent_path.write_text(
            json.dumps(_accepted_record(pending, reviewed_by=author_identity)),
            encoding="utf-8",
        )
        rejected = run_aw(
            root,
            "wi",
            "plan-review",
            "--evidence-file",
            str(not_independent_path),
            "--json",
            expect_success=False,
        )
        assert "not independent" in (rejected.stdout + rejected.stderr), (
            rejected.stdout,
            rejected.stderr,
        )

        accepted_path = root / "review-accepted.json"
        accepted_path.write_text(
            json.dumps(_accepted_record(pending, reviewed_by="independent-reviewer")),
            encoding="utf-8",
        )
        result = _last_json_value(
            run_aw(
                root,
                "wi",
                "plan-review",
                "--evidence-file",
                str(accepted_path),
                "--json",
            )
        )
        assert result["status"] == "accepted", result
        assert result["published_issue_count"] == len(candidates), result

        listing = _last_json_value(
            run_aw(root, "wi", "list", "--project", "demo", "--json")
        )
        published = [issue for issue in listing if issue.get("title") in titles]
        assert len(published) == len(candidates), (published, listing)
        published_titles = sorted(issue["title"] for issue in published)
        assert published_titles == titles, published_titles
        ids = {issue["id"] for issue in published}
        assert len(ids) == len(published), published

        # The decisive regression guard: both colliding candidate titles
        # persisted as distinct on-disk files (distinct slugs), not one
        # file silently overwritten by the second create.
        stems_by_title = _issue_stems_for_titles(root, titles)
        assert set(stems_by_title) == set(titles), (stems_by_title, titles)
        stems = set(stems_by_title.values())
        assert len(stems) == len(titles), stems_by_title

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()

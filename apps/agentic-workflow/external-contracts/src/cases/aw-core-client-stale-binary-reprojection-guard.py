"""Black-box contract for the `aw meta` stale-binary reprojection guard.

Drives the real `aw meta init` / `aw meta sync` / `aw meta check` commands
against a fixture whose declared checkout source version is provably ahead of
the installed binary, proving three independent things: (1) once a live
checkout copy of `CLAUDE.md.tmpl` exists, `aw meta sync` resolves the skew and
overwrites the managed block without needing any escape hatch; (2) with no
checkout copy available, a rewrite that would delete existing projection
content is refused (content left untouched) unless `--force-stale` is passed,
in which case the embedded-sourced rewrite proceeds; (3) `aw meta check`
routes the same binary-stale condition to a rebuild/upgrade remediation
(`cargo install --path apps/agentic-workflow` / `aw upgrade`) instead of
inviting the destructive `aw meta sync`.

The fixture never touches the real repository's own
`apps/agentic-workflow/Cargo.toml`; it plants a synthetic one (declaring
source version `99.99.99`, far ahead of the real installed binary) beneath an
isolated temporary root so the guard's "binary behind checkout" detection
fires deterministically without depending on this checkout's actual version.
`AW_ALLOW_STALE_BINARY=1` is set on every invocation purely to bypass the
*separate*, coarser stale-binary lifecycle-mutation gate
(`apps/agentic-workflow/src/cli/drift.rs::enforce_mutating_verb_gate`) that
would otherwise hard-refuse any lifecycle-mutating verb at this same
synthetic staleness before `aw meta`'s own finer-grained guard under test
ever runs; it is test plumbing, not part of the asserted promise.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-stale-binary-reprojection-guard"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "stale-binary-reprojection-guard"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-stale-binary-reprojection-guard"
)
ASSERTIONS = (
    "with no live checkout copy of CLAUDE.md.tmpl and the installed binary "
    "provably behind the checkout's declared source version, `aw meta sync` "
    "refuses a rewrite that would delete existing managed-block content: the "
    "file is left byte-for-byte untouched and the reported findings carry a "
    "`content_regression_blocked` blocker citing the source-version skew, "
    "even though the command itself still exits done rather than erroring",
    "landing a live checkout copy of CLAUDE.md.tmpl resolves the same "
    "provably-behind-checkout skew on its own: a second `aw meta sync` run "
    "against the same previously-blocked content, now with only the "
    "checkout copy added and still no `--force-stale`, actually overwrites "
    "the managed block -- proving the checkout copy is preferred over the "
    "binary's embedded snapshot and eliminates the guard window by itself",
    "passing `--force-stale` against the original no-checkout-copy skew "
    "lets the embedded-sourced rewrite proceed and remove the "
    "would-have-been-deleted content, and `aw meta check` on an equivalent "
    "still-skewed, still-blocked fixture reports status `binary_stale` with "
    "`next.command` naming the rebuild remediation "
    "(`cargo install --path apps/agentic-workflow`) and every finding's "
    "remediation text pointing at rebuild/`aw upgrade` and explicitly "
    "steering away from `aw meta sync` -- never the generic drift routing",
)

_STALE_SOURCE_CARGO_TOML = (
    '[package]\nname = "agentic-workflow"\nversion = "99.99.99"\n'
)
_INJECTED_PROSE = "PREVIOUSLY SYNCED PROSE"
_START_MARKER = "<!-- aw:start -->"
_CHECKOUT_TEMPLATE_RELATIVE = "apps/agentic-workflow/templates/cli/mainthread/CLAUDE.md.tmpl"
_MINIMAL_CHECKOUT_TEMPLATE = (
    "# CLAUDE.md Template (checkout copy)\n\n"
    "<!-- aw:start -->\n"
    "## Placeholder Section\n\n"
    "checkout-sourced adapter body\n\n"
    "### Workflow CLI\n\n"
    "<!-- aw:cli-table:workflow:start -->\n"
    "placeholder workflow table\n"
    "<!-- aw:cli-table:workflow:end -->\n\n"
    "### Support CLI\n\n"
    "<!-- aw:cli-table:support:start -->\n"
    "placeholder support table\n"
    "<!-- aw:cli-table:support:end -->\n"
    "<!-- aw:end -->\n"
)
_ENV = {"AW_ALLOW_STALE_BINARY": "1"}


def _plant_stale_source(root: Path) -> None:
    aw_dir = root / "apps" / "agentic-workflow"
    aw_dir.mkdir(parents=True, exist_ok=True)
    (aw_dir / "Cargo.toml").write_text(_STALE_SOURCE_CARGO_TOML, encoding="utf-8")


def _inject_prose(claude_path: Path) -> str:
    original = claude_path.read_text(encoding="utf-8")
    assert _START_MARKER in original, original
    injected = original.replace(
        _START_MARKER, f"{_START_MARKER}\n{_INJECTED_PROSE}", 1
    )
    claude_path.write_text(injected, encoding="utf-8")
    return injected


def verify() -> list[str]:
    with project_fixture() as root:
        _plant_stale_source(root)
        claude_path = root / "CLAUDE.md"

        init = run_aw(root, "meta", "init", env_overrides=_ENV)
        assert init.returncode == 0, init
        assert claude_path.exists(), "meta init must create CLAUDE.md"

        # -- phase 1: no checkout copy -> blocked without --force-stale -----
        with_prose = _inject_prose(claude_path)

        blocked = run_aw(root, "meta", "sync", env_overrides=_ENV)
        assert blocked.returncode == 0, blocked
        assert claude_path.read_text(encoding="utf-8") == with_prose, (
            "a blocked sync must leave the existing projection byte-for-byte "
            "untouched"
        )
        blocked_payload = final_json(blocked)
        blocked_findings = [
            finding
            for finding in blocked_payload["findings"]
            if finding["path"] == "CLAUDE.md"
        ]
        assert len(blocked_findings) == 1, blocked_payload
        skew_finding = blocked_findings[0]
        assert skew_finding["code"] == "content_regression_blocked", skew_finding
        assert skew_finding["severity"] == "blocker", skew_finding
        assert "would delete" in skew_finding["message"], skew_finding
        assert "behind checkout source v99.99.99" in skew_finding["message"], skew_finding
        assert "--force-stale" in skew_finding["remediation"], skew_finding

        # -- phase 2: landing a checkout copy resolves it without --force-stale
        checkout_template_path = root / _CHECKOUT_TEMPLATE_RELATIVE
        checkout_template_path.parent.mkdir(parents=True, exist_ok=True)
        checkout_template_path.write_text(
            _MINIMAL_CHECKOUT_TEMPLATE, encoding="utf-8"
        )

        resolved = run_aw(root, "meta", "sync", env_overrides=_ENV)
        assert resolved.returncode == 0, resolved
        resolved_content = claude_path.read_text(encoding="utf-8")
        assert _INJECTED_PROSE not in resolved_content, (
            "a live checkout copy must let the sync overwrite the managed "
            "block on its own, with no --force-stale needed"
        )
        resolved_payload = final_json(resolved)
        resolved_regressions = [
            finding
            for finding in resolved_payload["findings"]
            if finding["code"] == "content_regression_blocked"
        ]
        assert resolved_regressions == [], resolved_payload

        # -- phase 3: --force-stale proceeds without a checkout copy --------
        checkout_template_path.unlink()
        with_prose_again = _inject_prose(claude_path)
        assert with_prose_again != resolved_content, with_prose_again

        still_blocked = run_aw(root, "meta", "sync", env_overrides=_ENV)
        assert claude_path.read_text(encoding="utf-8") == with_prose_again, (
            "the guard must reapply once the checkout copy is gone again"
        )

        forced = run_aw(root, "meta", "sync", "--force-stale", env_overrides=_ENV)
        assert forced.returncode == 0, forced
        forced_content = claude_path.read_text(encoding="utf-8")
        assert _INJECTED_PROSE not in forced_content, (
            "--force-stale must let the embedded-sourced rewrite proceed"
        )
        forced_payload = final_json(forced)
        forced_regressions = [
            finding
            for finding in forced_payload["findings"]
            if finding["code"] == "content_regression_blocked"
        ]
        assert forced_regressions == [], forced_payload

        # -- phase 4: `aw meta check` routes binary-stale to rebuild/upgrade,
        #    never to `aw meta sync` --------------------------------------
        _inject_prose(claude_path)
        checked = run_aw(root, "meta", "check", env_overrides=_ENV, expect_success=False)
        assert checked.returncode != 0, checked
        checked_payload = final_json(checked)
        assert checked_payload["status"] == "binary_stale", checked_payload
        assert checked_payload["next"]["command"] == (
            "cargo install --path apps/agentic-workflow"
        ), checked_payload
        checked_finding = next(
            finding
            for finding in checked_payload["findings"]
            if finding["path"] == "CLAUDE.md"
            and finding["code"] == "content_regression_blocked"
        )
        assert "cargo install --path apps/agentic-workflow" in checked_finding[
            "remediation"
        ], checked_finding
        assert "aw upgrade" in checked_finding["remediation"], checked_finding
        assert "instead of `aw meta sync`" in checked_finding["remediation"], checked_finding

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()

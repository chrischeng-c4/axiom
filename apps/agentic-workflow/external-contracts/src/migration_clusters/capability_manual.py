"""Native Python ECs for capability projection and manual evidence artifacts."""

from __future__ import annotations

import time
import tomllib
from pathlib import Path
from typing import Any

from migration_clusters.work_item_planning import BOUNDED_BODY
from oracles import lumen_feature_class_reference as lumen_reference
from wi_contract_fixture import (
    REPOSITORY_ROOT,
    create,
    final_json,
    project_fixture,
    run_aw,
)


CASE_IDS = {
    "capability-control-plane-capability-project-sweep",
    "capability-control-plane-capability-readiness-reporting",
    "capability-control-plane-markdown-capability-schema",
    "capability-control-plane-missing-readme-initialization",
    "capability-control-plane-operational-efficiency",
    "capability-control-plane-operational-stability",
    "manual-evidence-schema-python-contract",
    "manual-runner-output-convention-python-contract",
    "manual-evidence-artifacts-operational-efficiency",
    "manual-evidence-artifacts-operational-stability",
}

CAPABILITY_DOCUMENT = """\
# Demo

## Brief

Demo capability fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Planning | - | implemented | verified | smoke | ready | verified; deterministic planning fixture |

### Planning

ID: planning
Type: DeveloperTool
Surfaces:
- CLI: `aw wi` - the planning fixture surface.
EC Dimensions:
- behavior: `true` - planning behavior fixture gate.
- efficiency: `true` - planning efficiency fixture gate.
- stability: `true` - planning stability fixture gate.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Provide deterministic planning.
Gate Inventory:
- tech-design/planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Planning ready | change | - | implemented | verified | smoke | `true` |
| Planning epic | epic | - | implemented | verified | smoke | `true` |
"""


def _capability_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        initialized = final_json(
            run_aw(
                root,
                "capability",
                "init",
                "--project",
                "demo",
                "--title",
                "Demo",
                "--brief",
                "Demo capability fixture.",
            )
        )
        cap_path = Path(initialized["cap_path"])
        shell = cap_path.read_text(encoding="utf-8")
        assert "## Brief" in shell
        assert "## Capabilities" in shell
        assert "### Capability Index" in shell
        # The two feature roots, in their canonical order. A fresh document that
        # renders no roots is one every new project would have to migrate before
        # it could classify anything, so "canonical shell" has to name them
        # rather than stop at the index.
        core_root = shell.find("### Core Features")
        non_core_root = shell.find("### Non-Core Features")
        assert core_root != -1, shell
        assert non_core_root != -1, shell
        assert shell.find("### Capability Index") < core_root < non_core_root, shell
        cap_path.write_text(CAPABILITY_DOCUMENT, encoding="utf-8")

        report = final_json(
            run_aw(
                root,
                "capability",
                "report",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        sweep = final_json(
            run_aw(
                root,
                "capability",
                "sweep",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        # Substring membership in a serialized blob would pass for any
        # implementation that echoes the project name, so the schema promise is
        # pinned to the parsed structure: one capability identity, its declared
        # type, every per-dimension gate command, and the exact ordered set of
        # Work Root claim slugs and their derived gate ids.
        capabilities = report["capabilities"]
        assert len(capabilities) == 1, capabilities
        capability = capabilities[0]
        assert capability["id"] == "planning", capability["id"]
        assert capability["title"] == "Planning", capability["title"]
        assert capability["capability_type"] == "DeveloperTool", capability
        assert capability["status"] == "verified", capability["status"]
        assert capability["promise"] == "Provide deterministic planning.", capability
        assert [
            (dimension["dimension"], dimension["runner"])
            for dimension in capability["ec_dimensions"]
        ] == [
            ("behavior", "true"),
            ("efficiency", "true"),
            ("stability", "true"),
        ], capability["ec_dimensions"]
        assert [claim["id"] for claim in capability["claims"]] == [
            "planning-ready",
            "planning-epic",
        ], capability["claims"]
        assert [
            (gate["id"], gate["command"]) for gate in capability["verification"]
        ] == [
            ("planning-ready-gate", "true"),
            ("planning-epic-gate", "true"),
        ], capability["verification"]
        assert capability["release_scope"] is True, capability
        assert report["capability_count"] == 1, report["capability_count"]
        assert report["claim_count"] == 2, report["claim_count"]
        # `Production | ready` in the index is a declaration, not evidence: no
        # claim gate has run, so readiness must still be false.
        assert capability["production_ready"] is False, capability
        assert capability["production_blockers"] == [
            "catalog/claim verification is not complete"
        ], capability["production_blockers"]
        assert [entry["project"] for entry in sweep["projects"]] == ["demo"], sweep
        swept = sweep["projects"][0]
        assert swept["report_status"] == report["status"], (swept, report["status"])
        assert swept["capability_count"] == report["capability_count"], swept
        assert swept["claim_count"] == report["claim_count"], swept
        assert swept["verified_claim_count"] == report["verified_claim_count"], swept
        return {"initialized": initialized, "report": report, "sweep": sweep}


def _lumen_report(root: Path, cap_path: Path, document: str) -> dict[str, Any]:
    cap_path.write_text(document, encoding="utf-8")
    return final_json(
        run_aw(
            root,
            "capability",
            "report",
            "--project",
            "demo",
            "--skip-issue-inventory",
        )
    )


def _lumen_feature_class_snapshot() -> dict[str, Any]:
    """Attribute a Lumen-shaped contract to its two feature roots.

    The reference document is asserted positively, then two falsifiers are
    asserted to be rejected: an implementation that ignores `Feature Class`, or
    reads only the field, or only the containing root, fails at least one.
    """
    before = lumen_reference.digest_production_contract(REPOSITORY_ROOT)
    with project_fixture() as root:
        initialized = final_json(
            run_aw(
                root,
                "capability",
                "init",
                "--project",
                "demo",
                "--title",
                "Lumen",
                "--brief",
                "Lumen reference fixture.",
            )
        )
        cap_path = Path(initialized["cap_path"])

        report = _lumen_report(root, cap_path, lumen_reference.REFERENCE_DOCUMENT)
        lumen_reference.assert_feature_class_attribution(report)

        # Every baseline the fixture names, not one representative: the claim is
        # that archetype baselines are always non-core.
        baseline_core = {}
        for cap_id in lumen_reference.NON_CORE_IDS:
            baseline_core[cap_id] = _lumen_report(
                root,
                cap_path,
                lumen_reference.baseline_declared_core_document(cap_id),
            )
            lumen_reference.assert_baseline_core_is_rejected(
                baseline_core[cap_id], cap_id
            )

        conflict = _lumen_report(
            root, cap_path, lumen_reference.ROOT_FIELD_CONFLICT_DOCUMENT
        )
        lumen_reference.assert_root_field_conflict_is_rejected(conflict)

        # The duplicate-root rule is the one whose blocker names neither the
        # field nor the class, so it is what proves the blocker filter subtracts
        # a pinned environment instead of admitting anticipated wordings.
        duplicate_root = _lumen_report(
            root, cap_path, lumen_reference.DUPLICATE_ROOT_DOCUMENT
        )
        lumen_reference.assert_duplicate_root_is_rejected(duplicate_root)

        # A bare duplicate heading is enough to put one capability under both
        # roots, so this rule is a single-message falsifier like the three above
        # rather than something needing a co-occurring-set assertion.
        multiply_classified = _lumen_report(
            root, cap_path, lumen_reference.MULTIPLY_CLASSIFIED_DOCUMENT
        )
        lumen_reference.assert_capability_under_both_roots_is_rejected(
            multiply_classified
        )

        # The other branch of the default-class rule: a legacy table parses to
        # zero capability sections, so the count comes from the rows and has to
        # be attributed from there. Nothing else in this fixture reaches it.
        legacy = _lumen_report(root, cap_path, lumen_reference.LEGACY_TABLE_DOCUMENT)
        lumen_reference.assert_legacy_rows_are_attributed_to_non_core(legacy)

        # Migration groups legacy *rows* through a branch separate from the one
        # that handles capability sections, so the derivation rule the section
        # legs bind could be inverted here unnoticed. The document is already
        # written, so migrate it in place.
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        legacy_migrated = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_legacy_migration_derives_the_split(legacy_migrated)
        lumen_reference.assert_migrated_legacy_index_lists_every_row(legacy_migrated)
        # The negative half: migration must emit a document the checker accepts,
        # not merely one containing the right substrings.
        legacy_migrated_report = final_json(
            run_aw(
                root,
                "capability",
                "report",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        lumen_reference.assert_migrated_legacy_document_is_accepted(
            legacy_migrated_report
        )

        # A retired capability leaves both classes and both totals. Without this
        # the word "retained" in every pair-sum assertion above is unearned:
        # nothing else the fixture runs carries a retired member.
        retired = _lumen_report(
            root, cap_path, lumen_reference.RETIRED_MEMBER_DOCUMENT
        )
        lumen_reference.assert_retired_capability_is_excluded_from_both_classes(retired)

        # Migration is the only path that has to *derive* the class instead of
        # reading one. Without this leg the derivation rule could be inverted
        # and every other assertion here would still pass, because every other
        # document states its own answer.
        # Reported *before* migrating it: a document that declares nothing must
        # read as wholly non-core and raise nothing. The default is the one
        # feature-class rule no other document here can exercise, because every
        # other one states its own answer.
        unclassified = _lumen_report(
            root, cap_path, lumen_reference.UNCLASSIFIED_DOCUMENT
        )
        lumen_reference.assert_unclassified_defaults_to_non_core(unclassified)

        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        migrated_text = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_migration_derives_the_split(migrated_text)
        migrated_report = final_json(
            run_aw(
                root,
                "capability",
                "report",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        # The migrated document must be one the checker accepts, and must carry
        # the same attribution as the hand-classified reference.
        lumen_reference.assert_feature_class_attribution(migrated_report)

        # The same unclassified shape with the two groups reversed. Migration
        # groups the sections but renders the index from a separate pass, and
        # the core-first input above cannot tell raw order from grouped order --
        # they are the same list. Non-core-first separates them, so the index
        # and the sections can actually disagree and be caught.
        cap_path.write_text(
            lumen_reference.NON_CORE_FIRST_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        fixed_point_text = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_migration_reaches_a_fixed_point(fixed_point_text)
        lumen_reference.assert_migration_derives_the_split(fixed_point_text)

        # Migration fills silence only. A class the author already stated has to
        # survive even where the derivation would have chosen the other one,
        # otherwise migration is a rewrite of the contract rather than a
        # completion of it.
        cap_path.write_text(
            lumen_reference.PARTIALLY_CLASSIFIED_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        preserved_text = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_migration_preserves_declared_class(preserved_text)

    after = lumen_reference.digest_production_contract(REPOSITORY_ROOT)
    lumen_reference.assert_production_contract_unmutated(before, after)
    return {
        "report": report,
        "baseline_core": baseline_core,
        "conflict": conflict,
        "duplicate_root": duplicate_root,
        "multiply_classified": multiply_classified,
        "legacy": legacy,
        "legacy_migrated": legacy_migrated_report,
        "retired": retired,
        "unclassified": unclassified,
        "migrated": migrated_report,
        "fixed_point": fixed_point_text,
        "preserved": preserved_text,
    }


def _manual_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        change = create(
            root,
            "Manual evidence fixture",
            "change",
            "--body",
            BOUNDED_BODY,
        )
        draft = final_json(
            run_aw(
                root,
                "ec",
                "draft",
                "manual-evidence",
                "--project",
                "demo",
                "--wi",
                change["slug"],
                "--capability-id",
                "planning",
                "--title",
                "Manual evidence fixture",
                "--json",
            )
        )
        pyproject = tomllib.loads((root / "external-contracts/pyproject.toml").read_text())
        inventory = pyproject["tool"]["aw"]["python-ec"]
        cases = inventory["cases"]
        assert cases[0]["id"] == "manual-evidence-behavior"
        assert cases[0]["evidence_paths"]
        runner = root / "external-contracts/src/runner.py"
        source = root / "external-contracts/src/manual-evidence.py"
        assert runner.is_file()
        assert source.is_file()
        assert draft["next"]["command"].startswith("aw ec check ")
        return {
            "draft": draft,
            "case": cases[0],
            "runner": runner.read_text(),
            "slug": change["slug"],
        }


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by capability-and-manual: {case_id}")
    started = time.monotonic()
    if case_id.startswith("capability-control-plane"):
        first = _capability_snapshot()
        if case_id == "capability-control-plane-capability-project-sweep":
            assertions = [
                "capability sweep groups the configured project and its next action",
                "the sweep retains the canonical planning capability identity",
            ]
        elif case_id == "capability-control-plane-capability-readiness-reporting":
            assertions = [
                "capability report resolves declared claim evidence",
                "project readiness is emitted from the canonical capability document",
            ]
        elif case_id == "capability-control-plane-markdown-capability-schema":
            _lumen_feature_class_snapshot()
            assertions = [
                "the field-style capability contract parses into one exact capability id, title, type, status, and promise",
                "every declared EC dimension keeps its exact gate command and every Work Root row becomes an exactly named claim and gate",
                "the sweep projection reports the same capability and claim counts and the same status as the report",
                "the Lumen reference fixture attributes its domain search promises to core and every archetype service baseline to non-core, with each per-class pair summing to its retained total",
                "each of the four trait-derived baselines is rejected as a blocker when declared core, naming that exact capability",
                "a Feature Class field contradicting its containing feature root is rejected as a blocker",
                "no document raises a blocker the fixture did not name, because document findings are separated from the scratch environment by subtraction rather than by a whitelist of wordings",
                "a feature root declared twice is rejected as a blocker whose message names neither the field nor the class, which is what the subtractive filter is required to see",
                "one capability listed under both feature roots is rejected as a blocker naming that exact capability, with every capability still parsing",
                "a document that declares no feature class at all raises no blocker and is attributed wholly to non-core, capabilities and claims alike, which is the default rule no self-describing document can exercise",
                "a legacy capability table is diagnosed as legacy and its rows are attributed wholly to non-core rather than falling out of both classes, which is the branch of that default rule where no capability section parses at all",
                "aw capability migrate derives the split for the rows of a legacy table through its own branch, placing the authored promises under Core Features and the trait-derived baseline under Non-Core Features, and the migrated document is accepted by a follow-up report with no blocker",
                "the migrated legacy document indexes every row it turned into a capability section, through the index branch legacy rows reach and the capability sections do not, so a migrated document cannot ship an empty table of contents alongside its sections",
                "a retired capability is excluded from both per-class counts and from the retained totals alike, so each per-class pair still sums against a total that genuinely excludes something",
                "aw capability migrate derives the split from an unclassified document, placing every authored promise under Core Features and every trait-derived baseline under Non-Core Features with the field and the containing root agreeing",
                "aw capability migrate reaches a fixed point on a non-core-first document: the migrated Capability Index and the migrated capability sections list the same capabilities in the same core-then-non-core order, so re-parsing the migrated document cannot render a different index again",
                "aw capability migrate preserves a class the author already declared, even where the derivation from the capability id would have chosen the other class",
                "Lumen's production capability contract is byte-identical before and after the fixture run",
            ]
        elif case_id == "capability-control-plane-missing-readme-initialization":
            assertions = [
                "capability init creates the missing canonical CAPABILITIES.md shell",
                "the shell contains Brief, Capabilities, and Capability Index sections, plus the Core Features and Non-Core Features roots in that order, so a fresh project starts classified-shaped rather than needing migration first",
            ]
        elif case_id == "capability-control-plane-operational-efficiency":
            assert time.monotonic() - started <= 120
            assertions = [
                "native capability init/report/sweep completes within 120 seconds",
                "all representative assertions pass without cargo delegation",
            ]
        else:
            second = _capability_snapshot()
            assert first["report"]["project"] == second["report"]["project"]
            assertions = [
                "two capability report/sweep executions preserve the same project identity",
                "both executions parse the canonical Markdown contract",
            ]
        return assertions

    first = _manual_snapshot()
    if case_id == "manual-evidence-schema-python-contract":
        # The generated inventory entry is deterministic generator output, so the
        # oracle is exact field-set equality rather than substring membership. A
        # generator that dropped `evidence_paths`, invented an oracle instead of
        # leaving the fill marker, or derived the evidence path from anything but
        # the case id would fail here.
        case = first["case"]
        assert case == {
            "id": "manual-evidence-behavior",
            "artifact_id": "artifact:demo/manual-evidence",
            "capability_id": "planning",
            "use_case_id": "manual-evidence",
            "dimension": "behavior",
            "applicability": "td",
            "test_path": "src/manual-evidence.py",
            "promise": "Manual evidence fixture",
            "oracle": "replace-with-independent-oracle",
            "target": "rust",
            "command": (
                "test -s external-contracts/evidence/manual-evidence-behavior.json"
            ),
            "evidence_paths": ["evidence/manual-evidence-behavior.json"],
        }, case
        return [
            "the generated manual EC inventory entry equals its exact declared field set",
            "the evidence path and gate command derive from the case id while the oracle stays an explicit fill marker",
        ]
    if case_id == "manual-runner-output-convention-python-contract":
        draft = first["draft"]
        assert draft["action"] == "python_ec_scaffold_created", draft["action"]
        assert draft["artifacts"] == [
            "external-contracts/pyproject.toml",
            "external-contracts/uv.lock",
            "external-contracts/src/runner.py",
            "external-contracts/src/manual-evidence.py",
        ], draft["artifacts"]
        assert draft["next"] == {
            "kind": "dispatch",
            "command": (
                f"aw ec check --project demo --json --wi {first['slug']}"
            ),
            "reason": (
                "author the generated Python EC source/inventory, then run its "
                "structural check"
            ),
            "payload_path": "external-contracts/pyproject.toml",
        }, draft["next"]
        # The scaffolded runner must refuse to run rather than exit zero with no
        # assertions, so an unfilled manual artifact can never read as evidence.
        assert "Python EC scaffold is incomplete" in first["runner"], first["runner"]
        return [
            "EC draft writes the runner, the case module, the inventory, and the lock in that exact artifact set",
            "the envelope emits the exact structural-check continuation and the scaffolded runner fails closed until authored",
        ]
    if case_id == "manual-evidence-artifacts-operational-efficiency":
        assert time.monotonic() - started <= 120
        return [
            "native Python EC scaffold/evidence gate completes within 120 seconds",
            "representative assertions pass without cargo delegation",
        ]
    second = _manual_snapshot()
    assert first["case"]["id"] == second["case"]["id"]
    assert first["case"]["evidence_paths"] == second["case"]["evidence_paths"]
    return [
        "two fresh EC scaffolds produce identical case and evidence identities",
        "both artifact envelopes route to the same structural check",
    ]

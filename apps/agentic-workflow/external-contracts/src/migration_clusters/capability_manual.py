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

        # The remaining two rules of `validate_capability_feature_roots`. Each
        # raises a co-occurring set rather than one message, which is an
        # assertion shape, not a reason to leave them unbound.
        missing_root = _lumen_report(
            root, cap_path, lumen_reference.MISSING_NON_CORE_ROOT_DOCUMENT
        )
        lumen_reference.assert_missing_non_core_root_is_rejected(missing_root)

        unknown_root = _lumen_report(
            root, cap_path, lumen_reference.UNKNOWN_FEATURE_ROOT_DOCUMENT
        )
        lumen_reference.assert_unknown_feature_root_is_rejected(unknown_root)

        # The other direction of the undeclared-class default: a *mistyped*
        # class must be refused outright rather than resolving to non-core the
        # way silence legitimately does. This document does not report at all,
        # so it is asserted against the failure.
        cap_path.write_text(
            lumen_reference.UNKNOWN_FEATURE_CLASS_DOCUMENT, encoding="utf-8"
        )
        refused = run_aw(
            root,
            "capability",
            "report",
            "--project",
            "demo",
            "--skip-issue-inventory",
            expect_success=False,
        )
        lumen_reference.assert_unknown_feature_class_value_is_refused(
            refused.returncode, refused.stderr
        )

        # The verified half of the split. Every leg above reads a report with no
        # gate execution, where all four verified fields are zero -- so the
        # pair-sum assertions covering them are `0 + 0 == 0` and hold for an
        # implementation that never attributes a verified capability at all.
        # Under `--verify` the counts are populated and the two classes differ in
        # both dimensions, which is what makes those four fields falsifiable.
        cap_path.write_text(lumen_reference.REFERENCE_DOCUMENT, encoding="utf-8")
        verified = final_json(
            run_aw(
                root,
                "capability",
                "report",
                "--project",
                "demo",
                "--skip-issue-inventory",
                "--verify",
            )
        )
        lumen_reference.assert_verified_split_is_non_degenerate(verified)

        # The human rendering of the split, which is built from its own format
        # string and which every other leg here is blind to because they all
        # read the JSON envelope. Driven on the verified report for the same
        # reason: four of that line's eight operands are verified counts, so on
        # an unverified report a transposition among them renders identically.
        human = run_aw(
            root,
            "capability",
            "report",
            "--project",
            "demo",
            "--skip-issue-inventory",
            "--verify",
            "--human",
        )
        lumen_reference.assert_human_report_renders_the_split(human.stdout, verified)

        # A third rendering of the same split, on a third surface. `aw capability
        # next` builds its `coverage` object from its own JSON literal rather
        # than from the report serializer, so it can zero or transpose a field
        # while every report-reading leg above stays green -- which is the exact
        # argument the `--human` leg makes, left unapplied here until now. The
        # reference document is still at `cap_path` from the verified leg.
        next_coverage = final_json(
            run_aw(
                root,
                "capability",
                "next",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )["coverage"]
        unverified_reference = _lumen_report(
            root, cap_path, lumen_reference.REFERENCE_DOCUMENT
        )
        lumen_reference.assert_next_coverage_matches_the_report(
            next_coverage, unverified_reference
        )

        # The second half of the effective-class rule. Every baseline leg above
        # states `Feature Class: core` and is rejected on the field; the class is
        # resolved as the field *or else* the containing root, and placement
        # alone must be enough. Without this the `or_else` half is unexercised
        # and a report that only ever read the field would pass every leg here.
        # Driven for every baseline, on the same reasoning as the declared-core
        # loop above: the claim is about archetype baselines as a set.
        placed_core = {}
        for cap_id in lumen_reference.NON_CORE_IDS:
            placed_core[cap_id] = _lumen_report(
                root, cap_path, lumen_reference.baseline_placed_core_document(cap_id)
            )
            lumen_reference.assert_baseline_placed_core_is_rejected(
                placed_core[cap_id], cap_id
            )

        # The accepting half of the `Feature Class` parser. The mistyped-value
        # leg above binds only the refusal; the spellings a human actually writes
        # -- backticked, hyphenated, camel-cased, and the root headings
        # themselves -- must resolve to the canonical pair, and that acceptance
        # is observable product behavior of `aw capability report` rather than an
        # implementation-internal rule.
        #
        # One document holds one spelling per capability, so the accepting set is
        # exercised in waves rather than in a single report. An earlier revision
        # ran one document and claimed the whole set; the `Core Features` /
        # `Non-Core Features` suffix family was unexercised and the claim was an
        # overclaim.
        human_spellings = {}
        for wave in range(lumen_reference.HUMAN_SPELLING_WAVES):
            human_spellings[wave] = _lumen_report(
                root, cap_path, lumen_reference.human_spelling_document(wave)
            )
            lumen_reference.assert_human_class_spellings_are_accepted(
                human_spellings[wave]
            )

        # `validate_capability_feature_roots` returns early unless the document
        # "declares any class", and that test is a three-way disjunction: a
        # capability field, a canonical root, or an unknown root. Every document
        # above satisfies the first arm, which masks the other two -- both were
        # independently deletable with the whole case still green.
        roots_only = _lumen_report(
            root, cap_path, lumen_reference.ROOTS_ONLY_DOCUMENT
        )
        lumen_reference.assert_roots_alone_classify_the_document(roots_only)

        unknown_roots_only = _lumen_report(
            root, cap_path, lumen_reference.UNKNOWN_ROOTS_ONLY_DOCUMENT
        )
        lumen_reference.assert_unknown_roots_alone_classify_the_document(
            unknown_roots_only
        )

        # Root membership scope. Every other document here places capability
        # headings strictly below their root and nothing else at the root's own
        # level, so the rule that a sibling heading *closes* the root is
        # unexercised -- and it is the one rule whose failure shows up as a
        # blocker that should not exist rather than one that is missing.
        sibling_heading = _lumen_report(
            root, cap_path, lumen_reference.SIBLING_HEADING_DOCUMENT
        )
        lumen_reference.assert_sibling_heading_closes_the_root(sibling_heading)

        # The product reads capability contracts in four Markdown forms and each
        # has its own `Feature Class` lookup. The canonical field-style section
        # is the only one every leg above drives; the other three were each
        # independently mutable to "never read the class" with the case green.
        alternate_forms = {}
        for form, document, expected_blockers in (
            ("field_value", lumen_reference.FIELD_VALUE_CONTRACT_DOCUMENT, []),
            ("contract_table", lumen_reference.CONTRACT_TABLE_DOCUMENT, []),
            (
                "yaml",
                lumen_reference.YAML_CONTRACT_DOCUMENT,
                [lumen_reference.YAML_CONTRACT_BLOCKER],
            ),
        ):
            alternate_forms[form] = _lumen_report(root, cap_path, document)
            lumen_reference.assert_alternate_form_reads_the_class(
                alternate_forms[form], form=form, expected_blockers=expected_blockers
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

        # `aw capability migrate` has a second entry point, and every leg above
        # drives only the first. When a project has no CAPABILITIES.md at all and
        # its README still carries the legacy table, migration *relocates* the
        # contract instead of reformatting one in place -- a different caller of
        # the same renderers, reached only from a fixture that was never
        # initialized. It needs its own project: this one has a CAPABILITIES.md
        # from the `capability init` above, which routes migration down the other
        # branch.
        with project_fixture() as relocation_root:
            relocation_readme = relocation_root / "README.md"
            relocation_cap = relocation_root / "CAPABILITIES.md"
            relocation_readme.write_text(
                lumen_reference.LEGACY_TABLE_DOCUMENT, encoding="utf-8"
            )
            assert not relocation_cap.exists(), (
                "the relocation branch fires only when no CAPABILITIES.md exists; "
                "this fixture must not be initialized first"
            )
            final_json(
                run_aw(relocation_root, "capability", "migrate", "--project", "demo")
            )
            assert relocation_cap.exists(), (
                "migration must create the relocated capability contract"
            )
            relocated_text = relocation_cap.read_text(encoding="utf-8")
            lumen_reference.assert_readme_relocation_preserves_tracker_state(
                relocation_readme.read_text(encoding="utf-8"), relocated_text
            )
            # And the relocated document must be one the checker accepts, not
            # merely one carrying the right substrings.
            relocated_report = final_json(
                run_aw(
                    relocation_root,
                    "capability",
                    "report",
                    "--project",
                    "demo",
                    "--skip-issue-inventory",
                )
            )
            lumen_reference.assert_migrated_legacy_document_is_accepted(
                relocated_report
            )

        # Relocation again, from the *other* input shape. A legacy table parses
        # to zero capability sections, so the leg above drives only
        # `render_capability_registry`'s legacy branch. A README whose contract
        # is canonical `###` sections parses into `document.capabilities` and
        # takes the other branch -- the one that renders the sections themselves
        # and that resolves each `Root WI` through `root_wi_for_capability` on
        # live, unblanked tracker state. Format migration cannot reach it either,
        # because it blanks that state before rendering. Three shapes, because
        # the branch has three distinguishable behaviors: nothing classified,
        # partially classified, and one class empty.
        relocated_sections = {}
        for name, readme_document in (
            ("unclassified", lumen_reference.UNCLASSIFIED_SECTION_README),
            ("partially_classified", lumen_reference.PARTIALLY_CLASSIFIED_SECTION_README),
        ):
            with project_fixture() as section_root:
                section_readme = section_root / "README.md"
                section_cap = section_root / "CAPABILITIES.md"
                section_readme.write_text(readme_document, encoding="utf-8")
                assert not section_cap.exists(), (
                    "the relocation branch fires only when no CAPABILITIES.md "
                    "exists; this fixture must not be initialized first"
                )
                final_json(
                    run_aw(section_root, "capability", "migrate", "--project", "demo")
                )
                assert section_cap.exists(), (
                    "migration must create the relocated capability contract"
                )
                section_text = section_cap.read_text(encoding="utf-8")
                relocated_sections[name] = final_json(
                    run_aw(
                        section_root,
                        "capability",
                        "report",
                        "--project",
                        "demo",
                        "--skip-issue-inventory",
                    )
                )
                lumen_reference.assert_relocation_preserves_section_tracker_state(
                    section_text
                )
                lumen_reference.assert_relocation_renders_every_capability_section(
                    section_text, relocated_sections[name]
                )

        # One canonical class with no members at all. Both roots must still be
        # emitted: a document missing a root is rejected by the product's own
        # checker, so "emit the roots that have members" would make migration
        # produce documents `aw capability report` refuses. No other input in
        # this fixture populates only one class.
        with project_fixture() as empty_class_root:
            empty_class_readme = empty_class_root / "README.md"
            empty_class_cap = empty_class_root / "CAPABILITIES.md"
            empty_class_readme.write_text(
                lumen_reference.ALL_CORE_SECTION_README, encoding="utf-8"
            )
            assert not empty_class_cap.exists()
            final_json(
                run_aw(empty_class_root, "capability", "migrate", "--project", "demo")
            )
            empty_class_text = empty_class_cap.read_text(encoding="utf-8")
            empty_class_report = final_json(
                run_aw(
                    empty_class_root,
                    "capability",
                    "report",
                    "--project",
                    "demo",
                    "--skip-issue-inventory",
                )
            )
            lumen_reference.assert_relocation_emits_both_roots_when_one_is_empty(
                empty_class_text, empty_class_report
            )

    after = lumen_reference.digest_production_contract(REPOSITORY_ROOT)
    lumen_reference.assert_production_contract_unmutated(before, after)
    return {
        "report": report,
        "baseline_core": baseline_core,
        "conflict": conflict,
        "duplicate_root": duplicate_root,
        "multiply_classified": multiply_classified,
        "missing_root": missing_root,
        "unknown_root": unknown_root,
        "legacy": legacy,
        "legacy_migrated": legacy_migrated_report,
        "retired": retired,
        "unclassified": unclassified,
        "migrated": migrated_report,
        "fixed_point": fixed_point_text,
        "preserved": preserved_text,
        "verified": verified,
        "relocated": relocated_report,
        "next_coverage": next_coverage,
        "placed_core": placed_core,
        "human_spellings": human_spellings,
        "roots_only": roots_only,
        "unknown_roots_only": unknown_roots_only,
        "sibling_heading": sibling_heading,
        "alternate_forms": alternate_forms,
        "relocated_sections": relocated_sections,
        "empty_class": empty_class_report,
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
                "the Lumen reference fixture attributes its domain search promises to core and every archetype service baseline to non-core, with the capability and claim pairs each summing to their retained total",
                "each of the four trait-derived baselines is rejected as a blocker when declared core, naming that exact capability",
                "a Feature Class field contradicting its containing feature root is rejected as a blocker",
                "no document raises a blocker the fixture did not name, because document findings are separated from the scratch environment by subtraction rather than by a whitelist of wordings",
                "a feature root declared twice is rejected as a blocker whose message names neither the field nor the class, which is what the subtractive filter is required to see",
                "one capability listed under both feature roots is rejected as a blocker naming that exact capability, with every capability still parsing",
                "deleting a canonical feature root is rejected together with every capability it stranded, asserted as the whole ordered blocker set rather than skipped for raising more than one message",
                "a feature root outside the closed pair is named as unknown rather than silently accepted, and is distinguished from the missing-root case by raising two blockers instead of five",
                "a Feature Class value outside the closed pair fails the command outright rather than resolving to non-core the way an undeclared class legitimately does",
                "aw capability report --verify attributes the verified capability and claim counts per class as well, on a report where those four fields are populated and the two classes differ in both, so the half of the split that is identically zero on an unverified report is falsifiable",
                "aw capability report --human renders the same core/non-core split the verified JSON envelope reports, with all eight operands non-zero and the two classes differing in every dimension so no transposition among them can pass",
                "a document that declares no feature class at all raises no blocker and is attributed wholly to non-core, capabilities and claims alike, which is the default rule no self-describing document can exercise",
                "a legacy capability table is diagnosed as legacy and its rows are attributed wholly to non-core rather than falling out of both classes, which is the branch of that default rule where no capability section parses at all",
                "aw capability migrate derives the split for the rows of a legacy table through its own branch, placing the authored promises under Core Features and the trait-derived baseline under Non-Core Features, and the migrated document is accepted by a follow-up report with no blocker",
                "the migrated legacy document indexes every row it turned into a capability section, through the index branch legacy rows reach and the capability sections do not, so a migrated document cannot ship an empty table of contents alongside its sections",
                "a retired capability is excluded from both per-class counts and from the retained totals alike, so each per-class pair still sums against a total that genuinely excludes something",
                "aw capability migrate derives the split from an unclassified document, placing every authored promise under Core Features and every trait-derived baseline under Non-Core Features with the field and the containing root agreeing",
                "aw capability migrate reaches a fixed point on a non-core-first document: the migrated Capability Index and the migrated capability sections list the same capabilities in the same core-then-non-core order, so re-parsing the migrated document cannot render a different index again",
                "aw capability migrate preserves a class the author already declared, even where the derivation from the capability id would have chosen the other class",
                "aw capability migrate relocating a README-resident legacy table into a project with no CAPABILITIES.md preserves each row's tracker state as its Root WI in both the index column and the section field, derives the same core/non-core split, and leaves the README a forwarding pointer instead of the table",
                "aw capability migrate relocating a README whose contract is canonical capability sections preserves each capability's own declared Root WI into both the index column and the section field, which is the branch that resolves it through root_wi_for_capability on live tracker state rather than through the legacy row or through format migration's pre-blanked input",
                "aw capability migrate renders one capability section per capability on that same branch, asserted through both the relocated text and a re-report of it, so a relocation emitting a complete-looking Capability Index over no contract at all cannot pass",
                "aw capability migrate emits both canonical feature roots when one class has no members, asserted on the only input shape where a populated-roots-only renderer differs from a both-roots renderer, and the emitted document is accepted by a follow-up report",
                "aw capability next renders the same core/non-core split its own report computes, through a coverage object built by a separate JSON literal, with the four populated operands non-zero and pairwise distinct",
                "each trait-derived baseline nested under Core Features while declaring no Feature Class at all is rejected by that exact blocker, which is the half of the effective-class rule that resolves the class from the containing root rather than from the field",
                "aw capability report resolves every human spelling of Feature Class -- backticked, hyphenated, camel-cased, case-folded, and the root headings themselves -- to its canonical class with the same per-class counts and no blocker, exercised in waves because one document holds one spelling per capability, which is the accepting half of the parser the mistyped-value assertion only binds the refusal of",
                "a document declaring no Feature Class field anywhere is still classified by its canonical feature roots alone, rejecting a baseline for its placement while every capability still reports the unclassified default, which is the arm of the declares-any-class test that a field-carrying document masks",
                "a document whose only feature roots are outside the closed pair is diagnosed rather than waved through as pre-migration, asserted as the whole ordered set of two missing-root and two unknown-root blockers, which is the third arm of that same test",
                "a heading at a feature root's own level closes that root, so a capability title repeated under a later sibling heading is not read as a member of both roots -- the one assertion here falsified by an implementation that reports too much rather than too little",
                "each of the three capability-contract reading forms other than the canonical field-style section -- the Field/Value contract, the one-row contract table, and the YAML-fenced section -- honours its Feature Class declaration, asserted per capability and by unequal per-class counts, because each form has its own class lookup and binding one binds none of the others",
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

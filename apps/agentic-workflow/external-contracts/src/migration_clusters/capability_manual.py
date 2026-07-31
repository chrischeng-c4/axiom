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

        # Every id above sits in both registries the rule unions, so the four
        # legs cannot tell the union from either half. These ids sit in exactly
        # one registry each, which is what makes dropping a registry observable.
        for cap_id in lumen_reference.REGISTRY_SPANNING_BASELINE_IDS:
            baseline_core[cap_id] = _lumen_report(
                root,
                cap_path,
                lumen_reference.registry_spanning_baseline_core_document(cap_id),
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
        # read the JSON envelope. Driven on partially verified reports, not on
        # the fully verified one: there `verified` and `total` are the same
        # integer in every dimension, so a line that read a total where a
        # verified count belongs renders exactly what a correct line renders.
        # Two shapes, because no single one makes all eight operands pairwise
        # distinct -- the reason is stated where the shapes are defined.
        partial = {}
        for (
            name,
            _claims,
            failing,
            operands,
        ) in lumen_reference.PARTIAL_VERIFICATION_SHAPES:
            cap_path.write_text(
                lumen_reference.PARTIALLY_VERIFIED_DOCUMENTS[name], encoding="utf-8"
            )
            partial[name] = final_json(
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
            lumen_reference.assert_partial_verification_is_attributed_per_class(
                partial[name], name, failing, operands
            )
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
            lumen_reference.assert_human_report_renders_the_split(
                human.stdout, partial[name]
            )

        # The reference document is what the legs below expect at `cap_path`.
        cap_path.write_text(lumen_reference.REFERENCE_DOCUMENT, encoding="utf-8")

        # A third rendering of the same split, on a third surface. `aw capability
        # next` builds its `coverage` object from its own JSON literal rather
        # than from the report serializer, so it can zero or transpose a field
        # while every report-reading leg above stays green -- which is the exact
        # argument the `--human` leg makes, left unapplied here until now. The
        # reference document was restored to `cap_path` above.
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

        # Root recognition is case-insensitive, and every document above writes
        # its roots in exact canonical case -- which cannot tell that tolerance
        # from a case-sensitive test. Driven once rather than per baseline: the
        # rule belongs to the heading parser, not to the baseline set, and the
        # loop above already sweeps that set at canonical case. The placed-core
        # shape is the one that makes it observable, because it is the shape
        # whose verdict depends on a root being read at all.
        case_varied_id = lumen_reference.NON_CORE_IDS[0]
        case_varied_roots = _lumen_report(
            root,
            cap_path,
            lumen_reference.case_varied_root_document(
                lumen_reference.baseline_placed_core_document(case_varied_id)
            ),
        )
        lumen_reference.assert_case_varied_roots_are_read_like_canonical_ones(
            case_varied_roots, placed_core[case_varied_id], case_varied_id
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

        # Fenced headings. The root scan masks lines inside code fences before
        # it parses any heading, and no other document here contains a fence at
        # all -- so deleting the mask left every leg green while a README that
        # merely *documents* the canonical shape would be reported as having a
        # duplicate root. Real Lumen and template READMEs carry exactly that.
        fenced_root = _lumen_report(
            root, cap_path, lumen_reference.FENCED_ROOT_DOCUMENT
        )
        lumen_reference.assert_fenced_headings_are_not_read_as_structure(fenced_root)

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
        # The rows arrive carrying `Active WI` values, so the legacy branch is
        # the one place a fixture can watch tracker state actually be dropped
        # rather than watch an already-empty field stay empty.
        lumen_reference.assert_migration_erases_legacy_row_tracker_state(
            legacy_migrated
        )
        # And each row's own remaining cells have to land in the section it
        # became. The leg above binds only the cells migration is required to
        # *drop*; two of the three it is required to carry were rewritable to
        # constants, which would collapse three distinct legacy capabilities
        # into three identical-looking sections.
        lumen_reference.assert_migrated_legacy_sections_carry_their_row_content(
            legacy_migrated,
            # Format migration erases document-stored tracker state, asserted
            # directly one leg above, so every section must render `Root WI: -`.
            tracker_state={
                title: "-" for title in lumen_reference.LEGACY_ROW_TRACKER_STATE
            },
        )
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

        # The same document under `--verify`. The leg above can only see two of
        # the four accumulators the retired filter guards: without gate
        # execution both verified counts are zero, so counting a retired item
        # into them was invisible. This is the same vacuity the `--verify` leg
        # fixed for the reference document, applied to the one document that
        # actually retires something.
        retired_verified = final_json(
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
        lumen_reference.assert_retired_is_excluded_from_the_verified_counts_too(
            retired_verified
        )

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
        # Deriving the class is not the whole rewrite. Every section is
        # re-rendered from the parsed capability, and of the fields the product's
        # own doc comment names as carried through untouched, only `ID:` was
        # bound: the promise, type, required verification, surfaces, EC
        # dimensions and gate inventory were each rewritable to one literal for
        # every capability in the document.
        lumen_reference.assert_sections_carry_their_own_contract(
            migrated_text, lumen_reference.UNCLASSIFIED_SECTION_TITLES
        )
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

        # The section branch's half of tracker-state erasure. Every document
        # above is authored the way `aw wi` leaves one -- `Root WI: -`, `-` in
        # every work-root cell -- so erasing tracker state on them is a no-op
        # that no assertion can distinguish from doing nothing. This document
        # carries live values in both places at once, which is what makes each
        # of the three assignments separately visible.
        cap_path.write_text(
            lumen_reference.LIVE_TRACKER_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        live_tracker_text = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_migration_erases_document_stored_tracker_state(
            live_tracker_text
        )
        # And the same derivation the unclassified leg binds still has to hold
        # on it, so erasure cannot be achieved by discarding the section.
        lumen_reference.assert_migration_derives_the_split(live_tracker_text)

        # Five of the Capability Index's seven columns were unbound. Every
        # document above writes the identical trailing cells for every row, so a
        # renderer that ignored `capability.index_summary` and printed one
        # constant row per capability produced the byte-identical table. This
        # input differs in every column and every row, which is what makes the
        # per-capability carry-through observable at all. It is *not*, as an
        # earlier revision claimed, the only branch on which `Production` is
        # reachable: the fallback renders that column for every capability of
        # every index-less document. What this branch alone reaches is a
        # `Production` value the product did not derive.
        cap_path.write_text(
            lumen_reference.VARIED_INDEX_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        varied_index_text = cap_path.read_text(encoding="utf-8")
        lumen_reference.assert_migration_carries_every_index_column(
            varied_index_text
        )
        lumen_reference.assert_migration_derives_the_split(varied_index_text)

        # The same index, written at the other heading level the parser accepts.
        # Every index-carrying document above writes `###`, so the level-2 arm
        # of that guard was free and an author's `## Capability Index` would be
        # read as no index at all.
        cap_path.write_text(
            lumen_reference.LEVEL_2_INDEX_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        lumen_reference.assert_migration_carries_every_index_column(
            cap_path.read_text(encoding="utf-8")
        )

        # And the same index with its `Notes` column removed, which is the only
        # input that reaches the promise fallback behind that cell: a blank cell
        # does not, because `table_cell` turns an empty cell into `-`.
        cap_path.write_text(
            lumen_reference.NO_NOTES_COLUMN_DOCUMENT, encoding="utf-8"
        )
        final_json(run_aw(root, "capability", "migrate", "--project", "demo"))
        lumen_reference.assert_migration_falls_back_to_the_promise_for_notes(
            cap_path.read_text(encoding="utf-8")
        )

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
            # Same renderer as the format-migration leg, reached from the other
            # entry point and -- unlike that one -- with tracker state live, so
            # the byte-exact block is asserted here against the rows' own WIs.
            # Binding it twice is what keeps `Root WI` a *read* rather than a
            # constant either caller happens to agree with.
            lumen_reference.assert_migrated_legacy_sections_carry_their_row_content(
                relocated_text,
                tracker_state=lumen_reference.LEGACY_ROW_TRACKER_STATE,
            )
            lumen_reference.assert_relocated_document_is_the_declared_frame(
                relocated_text
            )
            lumen_reference.assert_readme_residue_forwards_to_the_contract(
                relocation_readme.read_text(encoding="utf-8")
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
        # because it blanks that state before rendering. Four shapes, because
        # the branch has that many distinguishable behaviors: nothing
        # classified, partially classified, the `Root WI` fallback, and all
        # three render groups populated at once. The last one is what makes the
        # *order* of `capabilities_in_render_order` observable: in the other
        # three, grouped order and raw document order are the same list, so
        # permuting the array renders the identical document.
        relocated_sections = {}
        for name, readme_document, declares_any_class, expected_order in (
            (
                "unclassified",
                lumen_reference.UNCLASSIFIED_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "partially_classified",
                lumen_reference.PARTIALLY_CLASSIFIED_SECTION_README,
                True,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "work_root_wi",
                lumen_reference.WORK_ROOT_WI_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "mixed_class",
                lumen_reference.MIXED_SECTION_README,
                True,
                lumen_reference.MIXED_SECTION_GROUPED_TITLES,
            ),
            (
                "varied_status",
                lumen_reference.VARIED_STATUS_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "varied_work_root",
                lumen_reference.VARIED_WORK_ROOT_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "multi_item",
                lumen_reference.MULTI_ITEM_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "no_summary",
                lumen_reference.NO_SUMMARY_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "existing_pointer",
                lumen_reference.EXISTING_POINTER_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            (
                "derived_inventory",
                lumen_reference.DERIVED_INVENTORY_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
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
                if name == "work_root_wi":
                    # No capability declares a `Root WI`, so the renderer has to
                    # fall back to the first work root's WI. Every other shape
                    # here declares one, which leaves that branch unreachable.
                    lumen_reference.assert_relocation_falls_back_to_the_first_work_root_wi(
                        section_text
                    )
                else:
                    lumen_reference.assert_relocation_preserves_section_tracker_state(
                        section_text, expected_order=expected_order
                    )
                lumen_reference.assert_relocation_renders_every_capability_section(
                    section_text, relocated_sections[name], expected_order=expected_order
                )
                # Relocation re-renders every section, so the whole carried-
                # through field block is at risk on this path too -- and on a
                # different call site from the format-migration one above.
                lumen_reference.assert_sections_carry_their_own_contract(
                    section_text,
                    expected_order,
                    item_overrides={
                        "no_summary": lumen_reference.NO_SUMMARY_ITEM_OVERRIDES,
                        "derived_inventory": (
                            lumen_reference.DERIVED_INVENTORY_ITEM_OVERRIDES
                        ),
                    }.get(name),
                )
                if name == "derived_inventory":
                    # Every capability in every other document declares a gate
                    # inventory, so neither the derivation behind a missing one
                    # nor the `-` placeholder for an empty one was reachable.
                    lumen_reference.assert_relocation_derives_a_missing_gate_inventory(
                        section_text
                    )
                if name == "no_summary":
                    # Every other document declares both a command and a summary
                    # for every surface and dimension, so three of the four arms
                    # of each item renderer were never entered.
                    lumen_reference.assert_relocation_carries_a_command_only_item(
                        section_text
                    )
                # Relocation is a move, not a copy. Only the legacy-table shape
                # ever re-read the README it emptied, so on every section-shaped
                # input the residue write was unobservable and leaving the whole
                # contract behind in the README passed.
                lumen_reference.assert_section_relocation_empties_the_readme(
                    section_readme.read_text(encoding="utf-8"), expected_order
                )
                if name == "existing_pointer":
                    # The residue's early return. Every other input arrives
                    # without a `## Capability Contract` heading, so the branch
                    # that recognizes one -- and therefore idempotence across a
                    # second migrate run -- was never entered.
                    lumen_reference.assert_relocation_keeps_an_authored_contract_pointer(
                        section_readme.read_text(encoding="utf-8")
                    )
                if name == "varied_work_root":
                    # Every other document writes one constant work-root row for
                    # all eight roots, which leaves five of the row's seven cells
                    # replaceable by that constant.
                    lumen_reference.assert_relocation_carries_every_work_root_cell(
                        section_text
                    )
                if name == "multi_item":
                    # Every other document declares one item per list field, so
                    # rendering only the first element of each was
                    # indistinguishable from rendering all of them.
                    lumen_reference.assert_relocation_carries_every_list_item(
                        section_text
                    )
                if name == "varied_status":
                    # Every other document in this case is uniformly
                    # `Status: verified`, which makes the section's own status
                    # field, the two index columns derived from it, and the
                    # prose-prelude branch all unfalsifiable at once.
                    lumen_reference.assert_relocation_carries_per_capability_status(
                        section_text
                    )
                if name == "mixed_class":
                    # The only shape whose three render groups are all populated
                    # and whose grouped order differs from its input order, so
                    # it is the only one that can catch a permuted group array.
                    lumen_reference.assert_relocation_renders_the_three_groups_in_order(
                        section_text
                    )
                # The roots are emitted exactly when the input classified
                # something. Two of these shapes classify nothing and one
                # classifies its domain promises, so both directions are pinned.
                lumen_reference.assert_relocation_root_emission(
                    section_text, declares_any_class=declares_any_class
                )

        # A derivation with nothing at all to collect. Held apart from the loop
        # above because `aw capability report` rejects what `aw capability
        # migrate` writes for this input, while every shape in that loop is
        # asserted to report clean.
        with project_fixture() as empty_derivation_root:
            (empty_derivation_root / "README.md").write_text(
                lumen_reference.EMPTY_DERIVATION_SECTION_README, encoding="utf-8"
            )
            final_json(
                run_aw(
                    empty_derivation_root,
                    "capability",
                    "migrate",
                    "--project",
                    "demo",
                )
            )
            lumen_reference.assert_relocation_renders_an_underivable_gate_inventory(
                (empty_derivation_root / "CAPABILITIES.md").read_text(encoding="utf-8"),
                final_json(
                    run_aw(
                        empty_derivation_root,
                        "capability",
                        "report",
                        "--project",
                        "demo",
                        "--skip-issue-inventory",
                    )
                ),
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
        "retired_verified": retired_verified,
        "fenced_root": fenced_root,
        "unclassified": unclassified,
        "migrated": migrated_report,
        "fixed_point": fixed_point_text,
        "live_tracker": live_tracker_text,
        "preserved": preserved_text,
        "verified": verified,
        "partial": partial,
        "relocated": relocated_report,
        "next_coverage": next_coverage,
        "placed_core": placed_core,
        "case_varied_roots": case_varied_roots,
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
                "the same rejection holds for baselines the fixture does not otherwise carry, chosen to span both registries the baseline set unions -- one supplied only by the capability families and two supplied only by the archetype traits -- so deleting either registry is observable rather than masked by ids that both supply",
                "a Feature Class field contradicting its containing feature root is rejected as a blocker",
                "no document raises a blocker the fixture did not name, because document findings are separated from the scratch environment by subtraction rather than by a whitelist of wordings",
                "a feature root declared twice is rejected as a blocker whose message names neither the field nor the class, which is what the subtractive filter is required to see",
                "one capability listed under both feature roots is rejected as a blocker naming that exact capability, with every capability still parsing",
                "deleting a canonical feature root is rejected together with every capability it stranded, asserted as the whole ordered blocker set rather than skipped for raising more than one message",
                "a feature root outside the closed pair is named as unknown rather than silently accepted, and is distinguished from the missing-root case by raising two blockers instead of five",
                "a Feature Class value outside the closed pair fails the command outright rather than resolving to non-core the way an undeclared class legitimately does",
                "aw capability report --verify attributes the verified capability and claim counts per class as well, on a report where those four fields are populated and the two classes differ in both, so the half of the split that is identically zero on an unverified report is falsifiable",
                "aw capability report --human renders the same core/non-core split the JSON envelope of the same run reports, driven on partially verified reports where each of the eight operands is non-zero, the two classes differ in every dimension, and every verified count falls strictly short of its own total, so neither a transposition across the classes nor a total rendered in place of its own verified count can pass",
                "an unverified claim is subtracted from its own class rather than from the other, asserted as the eight readiness operands pinned to exact integers across two partial-verification shapes whose per-operand collisions do not overlap, so no pair of operands agrees in every document the split is asserted on, and the failure is reported against the capability that owns each unverified claim",
                "a document that declares no feature class at all raises no blocker and is attributed wholly to non-core, capabilities and claims alike, which is the default rule no self-describing document can exercise",
                "a legacy capability table is diagnosed as legacy and its rows are attributed wholly to non-core rather than falling out of both classes, which is the branch of that default rule where no capability section parses at all",
                "aw capability migrate derives the split for the rows of a legacy table through its own branch, placing the authored promises under Core Features and the trait-derived baseline under Non-Core Features, and the migrated document is accepted by a follow-up report with no blocker",
                "the migrated legacy document indexes every row it turned into a capability section, through the index branch legacy rows reach and the capability sections do not, so a migrated document cannot ship an empty table of contents alongside its sections",
                "the migrated legacy document carries none of the Active WI values its rows arrived with, asserted through the index column, every rendered Root WI field, and the absence of the raw values from the whole document, so the delivery-provenance-is-one-way rule is bound on the one input branch whose rows actually supply tracker state",
                "a retired capability is excluded from both per-class counts and from the retained totals alike, so each per-class pair still sums against a total that genuinely excludes something",
                "aw capability migrate derives the split from an unclassified document, placing every authored promise under Core Features and every trait-derived baseline under Non-Core Features with the field and the containing root agreeing",
                "aw capability migrate reaches a fixed point on a non-core-first document: the migrated Capability Index and the migrated capability sections list the same capabilities in the same core-then-non-core order, so re-parsing the migrated document cannot render a different index again",
                "aw capability migrate erases the tracker state a capability section stored, asserted on the one document carrying both a live Root WI field and a live work-root WI at once -- so the field, the work-root cell, and the gap fallback that root_wi_for_capability reads when the field is blank are each separately observable -- while the same document still derives its split and keeps its gate inventory",
                "aw capability migrate preserves a class the author already declared, even where the derivation from the capability id would have chosen the other class",
                "aw capability migrate relocating a README-resident legacy table into a project with no CAPABILITIES.md preserves each row's tracker state as its Root WI in both the index column and the section field, derives the same core/non-core split, and leaves the README a forwarding pointer instead of the table",
                "aw capability migrate relocating a README whose contract is canonical capability sections preserves each capability's own declared Root WI into both the index column and the section field, which is the branch that resolves it through root_wi_for_capability on live tracker state rather than through the legacy row or through format migration's pre-blanked input",
                "aw capability migrate renders one capability section per capability on that same branch, asserted through both the relocated text and a re-report of it, so a relocation emitting a complete-looking Capability Index over no contract at all cannot pass",
                "aw capability migrate renders core, then non-core, then whatever declared nothing, asserted on the one relocation shape whose three render groups are all populated and whose grouped order differs from its input order, with the Capability Index and the capability sections -- two separate passes over the same group array -- pinned both to each other and to that grouped order",
                "aw capability migrate emits both canonical feature roots when one class has no members, asserted on the only input shape where a populated-roots-only renderer differs from a both-roots renderer, and the emitted document is accepted by a follow-up report",
                "aw capability next renders the same core/non-core split its own report computes, through a coverage object built by a separate JSON literal, with the four populated operands non-zero and pairwise distinct",
                "each trait-derived baseline nested under Core Features while declaring no Feature Class at all is rejected by that exact blocker, which is the half of the effective-class rule that resolves the class from the containing root rather than from the field",
                "the same placement is read the same way when both feature roots are written in a different case, asserted as the identical blocker and the identical class-partitioned counts as its canonical-cased twin, because an unrecognized root does not misclassify but disappears -- and every other document here writes its roots in exact canonical case",
                "aw capability report resolves every human spelling of Feature Class -- backticked, hyphenated, camel-cased, case-folded, and the root headings themselves -- to its canonical class with the same per-class counts and no blocker, exercised in waves because one document holds one spelling per capability, which is the accepting half of the parser the mistyped-value assertion only binds the refusal of",
                "a document declaring no Feature Class field anywhere is still classified by its canonical feature roots alone, rejecting a baseline for its placement while every capability still reports the unclassified default, which is the arm of the declares-any-class test that a field-carrying document masks",
                "a document whose only feature roots are outside the closed pair is diagnosed rather than waved through as pre-migration, asserted as the whole ordered set of two missing-root and two unknown-root blockers, which is the third arm of that same test",
                "a heading at a feature root's own level closes that root, so a capability title repeated under a later sibling heading is not read as a member of both roots -- the one assertion here falsified by an implementation that reports too much rather than too little",
                "each of the three capability-contract reading forms other than the canonical field-style section -- the Field/Value contract, the one-row contract table, and the YAML-fenced section -- honours its Feature Class declaration, asserted per capability and by unequal per-class counts, because each form has its own class lookup and binding one binds none of the others",
                "a heading inside a Markdown code fence is neither a feature root nor a member of one, asserted on a document whose fenced appendix would otherwise duplicate a root and place one capability under both, which is the shape real project READMEs carry",
                "a relocated capability with no declared Root WI falls back to its first work root's WI in both the index column and the section field, with the second work root's WI appearing nowhere, which is the branch of that resolution every Root-WI-declaring input leaves unreachable",
                "aw capability migrate emits the two canonical feature roots exactly when its input classified something, asserted in both directions across relocation shapes that differ in that one property, so neither an unconditional renderer nor one that never emits them can pass",
                "a retired capability is excluded from the verified capability and verified claim counts as well, asserted under --verify where those two accumulators are populated and the classes differ in both, which is the half of the retired filter an unverified report holds vacuously",
                "each legacy row's own Current State, Gaps, and Evidence land in the capability section it becomes, asserted as the whole byte-exact section body per row -- separator included, an earlier revision having compared after stripping trailing newlines on the ground that the blank lines before the next heading were pinned elsewhere, which they were not -- against pairwise-distinct cells rather than as substrings, and bound on both entry points -- format migration, where document-stored tracker state is erased and every Root WI must therefore render `-`, and README relocation, where it is live and each row's own WI must render -- so no field of the rendered section can be a constant one of the two callers happens to agree with",
                "every rendered capability section carries its own Promise, Type, Required Verification, Surfaces, EC Dimensions, Gate Inventory, and Dependencies rather than a shared one, asserted on both re-rendering paths -- format migration and README relocation -- against values made pairwise distinct per capability, down to the surface kind and the EC dimension kind, which are separate reads from the command and summary beside them and stayed constant while the assembled item varied",
                "the Dependencies field is asserted in both directions, present for the two capabilities that declare one and absent for the four that do not, because no capability declared one at all and the whole block was deletable while the product's own carry-through comment names product dependencies, with one of the two declaring more than one dependency, out of sorted order and with a repeat, and asserted as the whole rendered block, because both declaring capabilities previously carried exactly one and rendering only the first left the loop, the sort, and the deduplication of that parse all rendering the identical document",
                "a capability's Surfaces, EC Dimensions, and Gate Inventory keep every item in declaration order, asserted on the one input where a capability declares two of each, because every other document declares exactly one item per list and rendering only the first element of each is byte-identical on those",
                "every cell of every work-root row survives relocation, asserted on the one input whose eight rows differ in Kind, Impl, Verification, Maturity, and Gate / Evidence, so none of those five cells can be the constant the other inputs all happen to write",
                "the Capability Index Maturity column is asserted on the relocation branch as well, against each capability's own Required Verification, because that branch derives it rather than carrying it and the derivation stopped being constant once the fixture varied the field it reads",
                "a promise containing a pipe is escaped into the Capability Index Notes cell it falls back into, asserted through a row reader that splits on unescaped pipes only, so an unescaped pipe adds a column and fails to parse rather than being silently absorbed",
                "the README a section-shaped capability contract was relocated out of keeps only a forwarding pointer and keeps everything that was never part of that contract, asserted on every section-shaped relocation, so relocation can neither leave a second divergent copy of the contract behind nor truncate the README around it",
                "every Capability Index cell a capability arrived with is carried through per capability, asserted across all five non-identity columns on an input that differs in every column and every row, which is the branch on which a Production value the product did not derive is reachable -- the derived Production and Maturity values are reachable on the index-less branch too and are asserted there separately",
                "a relocated capability keeps its own Status, the prose prelude above its fields, and the Impl and Verification columns derived from that status, asserted on the one input whose capabilities are not uniformly verified -- three distinct derived pairs across four statuses, so neither column can be a constant",
                "the Capability Index header row and its alignment row are asserted as exact literals, because every reader of that table finds its columns by name and would keep passing against a renamed column or a moved right-alignment",
                "the derived Production column is asserted as `not_ready` for every capability of every index-less document -- six capabilities across five documents -- because that derivation runs on every such document and its answer was read back by nothing",
                "the Capability Index is recognized at either heading level the parser accepts, asserted on a `##` index whose columns come through identically to the `###` index every other document here writes, so the level-2 arm of that guard is not free",
                "a Capability Index that declares no Notes column at all falls back to each capability's own promise in that cell, which is the only input that reaches the fallback -- a blank cell does not, because an empty cell is read as `-` -- with the four columns the document still declares carried through unchanged",
                "the Root WI fallback is asserted against every spelling the product treats as an empty table value, one spelling per capability, so a fallback that recognizes only the literal `-` leaves the others standing as rendered tracker state",
                "the document relocation creates is asserted as its whole declared frame -- the project title, Brief, the machine-readable-contract note, and the Capabilities heading in that order -- against a project name that appears nowhere in the input, so a frame that dropped a heading, reordered them, or hard-coded the title cannot pass",
                "the forwarding pointer left in the emptied README is asserted as its exact block including the relative link to CAPABILITIES.md, because a pointer that names the contract without linking to it is not a pointer",
                "a README that already carries an authored `## Capability Contract` heading keeps it verbatim and gains no second pointer, which is the early return in the residue renderer that every other input leaves unentered and the reason a second migrate run is idempotent",
                "a surface and an EC dimension that declare a command but no summary render the command-only form, asserted on the one input that declares one, because every other document declares both halves for every item and three of the four arms of each item renderer were never entered",
                "a capability that declared no gate inventory gets the one its claims imply, asserted as the exact item list of the rendered field across four capabilities of the same document -- one deriving a single gate, one deriving two refs from two work roots so that joining the list is distinguishable from keeping only its first or only its last element, whose two refs are drawn one from each half of the derivation and declared in the reverse of the order they must render in, so that collecting the claim fixtures before the capability gates is distinguishable from collecting them in work-root order, one whose work-root cell is not backticked and therefore derives through the claim-fixture half of the derivation rather than the claim-gate half, and one declaring only empty-table spellings that gets the single `-` placeholder with its own work-root gate not derived in behind it",
                "a capability with no declared gate inventory and nothing to derive one from renders the placeholder through the derivation's own empty arm, asserted on its own document because the document `aw capability migrate` writes for that input is one `aw capability report` then rejects, named as the exact claim that has neither a gate nor a fixture",
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

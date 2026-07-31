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
- behavior: `gate-behavior` - planning behavior fixture gate.
- efficiency: `gate-efficiency` - planning efficiency fixture gate.
- stability: `gate-stability` - planning stability fixture gate.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Provide deterministic planning.
Gate Inventory:
- tech-design/planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Planning ready | change | - | implemented | verified | smoke | `gate-ready` |
| Planning epic | epic | - | implemented | verified | smoke | `gate-epic` |
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
        #
        # The five commands in the document -- three EC-dimension runners and
        # two Work Root Gate/Evidence cells -- are pairwise distinct on purpose.
        # Until round 32 all five were the same string `true`, which bound the
        # dimension *names* and the gate *ids* while leaving the command-to-key
        # binding free: an implementation emitting one constant runner, or
        # transposing runners across dimensions, passed both tuple comparisons
        # unchanged. Distinct values are what make the pairing observable.
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
            ("behavior", "gate-behavior"),
            ("efficiency", "gate-efficiency"),
            ("stability", "gate-stability"),
        ], capability["ec_dimensions"]
        assert [claim["id"] for claim in capability["claims"]] == [
            "planning-ready",
            "planning-epic",
        ], capability["claims"]
        assert [
            (gate["id"], gate["command"]) for gate in capability["verification"]
        ] == [
            ("planning-ready-gate", "gate-ready"),
            ("planning-epic-gate", "gate-epic"),
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
        # Two of these four comparisons used to read `swept[x] == report[x]`,
        # and two of those could not fail.
        #
        # `report.status` is forced to "blocked" whenever the report carries any
        # blocker, and a scratch project always carries the Python EC and TD
        # inventory blockers, so the status is a constant in this fixture: an
        # implementation that hardcoded `report_status: "blocked"` in
        # `capability_sweep_project` passed a self-comparison. Both sides are
        # pinned to the literal instead, which at least refutes a projection
        # that reports some *other* constant.
        #
        # What that still does not bind, and nothing in this case does, is the
        # branch where the two legitimately disagree: on current successful
        # verify evidence the projection overwrites `report_status` to
        # "healthy", flips `loop_status` to "done", and raises both verified
        # counts to their totals. Reaching it needs a fixture project with real
        # Python EC and TD manifests and committed verify evidence matching the
        # fixture's own git HEAD -- a whole scratch repository, not a document.
        # It is disclosed rather than claimed.
        assert report["status"] == "blocked", report["status"]
        assert swept["report_status"] == "blocked", swept
        # These two are anchored: `capability_count` and `claim_count` are
        # pinned to literals above, so the equality carries those literals
        # through to the projection.
        assert swept["capability_count"] == report["capability_count"], swept
        assert swept["claim_count"] == report["claim_count"], swept
        # `verified_claim_count` has no such anchor -- no claim gate has run, so
        # both sides are zero and the equality is `0 == 0`. Pinned to the
        # literal for the same reason as the status.
        assert report["verified_claim_count"] == 0, report
        assert swept["verified_claim_count"] == 0, swept
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
        lumen_reference.assert_migration_index_and_sections_agree_on_order(
            fixed_point_text
        )
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
                "partial_item",
                lumen_reference.PARTIAL_ITEM_SECTION_README,
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
            # A surface `kind` and an EC dimension `kind` are each parsed through
            # a fold table that accepts several spellings, and every other
            # document here writes only the spelling the table returns. That left
            # every alias arm of both tables deletable without changing a
            # rendered byte -- and a missing EC dimension arm drops the dimension
            # silently rather than carrying it through unfolded.
            (
                "alias_spellings",
                lumen_reference.ALIAS_SPELLING_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            # Every document above gives every capability a work-root table, so
            # only the first of the four blocks that render one was ever
            # entered. The three that synthesize a row for a capability with no
            # work roots were all deletable together.
            (
                "no_work_root",
                lumen_reference.NO_WORK_ROOT_SECTION_README,
                False,
                lumen_reference.UNCLASSIFIED_SECTION_TITLES,
            ),
            # Every document above declares at most one EC dimension item per
            # kind, so the map that collapses them never merged anything and
            # both of its field fills were deletable. Two capabilities here
            # split one dimension across two half-items, in opposite orders, so
            # each fill is the sole reason for its own merged item.
            (
                "same_kind_ec_dimension",
                lumen_reference.SAME_KIND_EC_DIMENSION_SECTION_README,
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
                        section_text,
                        expected_order=expected_order,
                        # One capability of the no-work-root document declares
                        # no tracker state at all, which is what makes the
                        # last-resort synthesized row reachable.
                        blanked_titles=(
                            lumen_reference.NO_WORK_ROOT_BLANKED_TITLES
                            if name == "no_work_root"
                            else frozenset()
                        ),
                    )
                lumen_reference.assert_relocation_renders_every_capability_section(
                    section_text,
                    relocated_sections[name],
                    expected_order=expected_order,
                    # Only the varied-status document declares a `retired`
                    # capability, and the report's totals exclude it. Passed per
                    # document rather than derived from the report, so a product
                    # that stopped excluding it fails here instead of being
                    # accommodated.
                    retired_titles=(
                        lumen_reference.VARIED_STATUS_RETIRED_TITLES
                        if name == "varied_status"
                        else frozenset()
                    ),
                )
                # Relocation re-renders every section, so the whole carried-
                # through field block is at risk on this path too -- and on a
                # different call site from the format-migration one above.
                lumen_reference.assert_sections_carry_their_own_contract(
                    section_text,
                    expected_order,
                    item_overrides={
                        "partial_item": lumen_reference.PARTIAL_ITEM_OVERRIDES,
                        "derived_inventory": (
                            lumen_reference.DERIVED_INVENTORY_ITEM_OVERRIDES
                        ),
                        "multi_item": lumen_reference.MULTI_ITEM_OVERRIDES,
                        # Not an authored shape: the efficiency merge appends a
                        # dimension of its own to one capability here.
                        "varied_status": lumen_reference.VARIED_STATUS_OVERRIDES,
                        # The kinds are authored as aliases, so the rendered
                        # items are the canonical spellings -- and the EC
                        # dimensions render in enum order rather than the order
                        # the document writes them.
                        "alias_spellings": lumen_reference.ALIAS_SPELLING_OVERRIDES,
                    }.get(name),
                )
                if name == "derived_inventory":
                    # Every capability in every other document declares a gate
                    # inventory, so neither the derivation behind a missing one
                    # nor the `-` placeholder for an empty one was reachable.
                    lumen_reference.assert_relocation_derives_a_missing_gate_inventory(
                        section_text
                    )
                    # The migrated field is the *union* of a capability's refs,
                    # so which claim each came from, what gate id it was given,
                    # and which work root it proves are invisible in it. Read off
                    # the report instead -- and the id of a claim's second gate
                    # was unreachable until one work root declared two.
                    lumen_reference.assert_derived_claims_carry_their_own_gates(
                        relocated_sections[name]
                    )
                if name == "partial_item":
                    # Every other document declares both a command and a summary
                    # for every surface and dimension, so three of the four arms
                    # of each item renderer were never entered. All three partial
                    # shapes live here, on three different capabilities -- one
                    # arm each, so no section's assertion answers for two.
                    lumen_reference.assert_relocation_carries_a_command_only_item(
                        section_text
                    )
                    lumen_reference.assert_relocation_carries_a_summary_only_item(
                        section_text
                    )
                    # The neither-half arm. It renders the bare kind, so blanking
                    # it does not shorten the item, it deletes the only thing
                    # naming what was declared.
                    lumen_reference.assert_relocation_carries_a_bare_item(section_text)
                # Relocation is a move, not a copy. Only the legacy-table shape
                # ever re-read the README it emptied, so on every section-shaped
                # input the residue write was unobservable and leaving the whole
                # contract behind in the README passed.
                lumen_reference.assert_section_relocation_empties_the_readme(
                    section_readme.read_text(encoding="utf-8"), expected_order
                )
                if name == "no_work_root":
                    lumen_reference.assert_relocation_synthesizes_an_absent_work_root_table(
                        section_text
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
                    # And every assertion on this renderer reads one field at a
                    # time, so the order the fields are emitted in was unbound:
                    # reversing the four conditional field blocks left all of
                    # them green. This document is where the whole block is
                    # compared as one string, because it is the one that carries
                    # both a multi-item capability and a dependent one.
                    lumen_reference.assert_relocation_renders_the_canonical_field_block(
                        section_text, lumen_reference.CANONICAL_BLOCK_SUBJECTS
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
                    # `Feature Class` renders in the *middle* of the canonical
                    # field block, and no subject in `multi_item` declares one,
                    # so the block comparison there binds its absence only. This
                    # document is the only one that classifies, so it is where a
                    # present class -- of both values -- gets its position bound.
                    lumen_reference.assert_relocation_renders_the_canonical_field_block(
                        section_text, lumen_reference.MIXED_CANONICAL_BLOCK_SUBJECTS
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

        # The frame the *format* migration has to supply. Every document above
        # arrives with a title, a Brief, and a Capabilities heading already, so
        # all three repairs could be disabled at once without changing a
        # rendered byte. Each input here is missing exactly one of them, and the
        # brief repair's two arms -- promote the lead prose, or write the
        # human-confirmation placeholder -- get one input each.
        for frame_label, (
            frame_document,
            frame_expected,
        ) in lumen_reference.FRAME_REPAIRS.items():
            with project_fixture() as frame_root:
                (frame_root / "README.md").write_text(
                    "# Demo\n\nUnrelated to the capability contract.\n",
                    encoding="utf-8",
                )
                frame_cap = frame_root / "CAPABILITIES.md"
                frame_cap.write_text(frame_document, encoding="utf-8")
                final_json(
                    run_aw(frame_root, "capability", "migrate", "--project", "demo")
                )
                lumen_reference.assert_format_migration_repairs_the_canonical_frame(
                    frame_cap.read_text(encoding="utf-8"),
                    expected=frame_expected,
                    label=frame_label,
                )

        # A project whose capability contract has not been written yet. Every
        # other document here declares either a capability section or a legacy
        # row, so the guard that keeps an empty registry from acquiring two
        # memberless feature roots was only ever entered with something to
        # render. The input carries a legacy level-2 `## Capability Index`
        # because that is what makes migration run at all on a contract-less
        # document; a bare `## Capabilities` heading returns "already
        # canonical" before the renderer is reached.
        with project_fixture() as empty_registry_root:
            (empty_registry_root / "README.md").write_text(
                "# Demo\n\nUnrelated to the capability contract.\n", encoding="utf-8"
            )
            empty_registry_cap = empty_registry_root / "CAPABILITIES.md"
            empty_registry_cap.write_text(
                lumen_reference.EMPTY_REGISTRY_DOCUMENT, encoding="utf-8"
            )
            empty_registry = final_json(
                run_aw(
                    empty_registry_root, "capability", "migrate", "--project", "demo"
                )
            )
            lumen_reference.assert_an_empty_registry_gains_no_feature_roots(
                empty_registry,
                empty_registry_cap.read_text(encoding="utf-8"),
            )

        # Convergence of the migrate tick loop. Every leg above migrates exactly
        # once and reads the result, so a migration that rewrote its input on
        # every invocation -- or that reported a no-op while still writing --
        # satisfied all of them, and would keep satisfying them while
        # `aw capability migrate` never terminated for an adopter driving it to
        # completion. Driven on both arrival paths, because the guard that
        # answers "already canonical" is a conjunction of two conditions and
        # each subject leaves one of them free: a README-resident contract is
        # relocated and then feature-class migrated, a resident classified
        # document takes the format phase alone.
        for subject, idempotence_readme, idempotence_document, expected_ticks in (
            (
                "README relocation",
                lumen_reference.MULTI_ITEM_SECTION_README,
                None,
                lumen_reference.MIGRATION_TICKS_FROM_README,
            ),
            (
                "resident non-core-first document",
                None,
                lumen_reference.NON_CORE_FIRST_DOCUMENT,
                lumen_reference.MIGRATION_TICKS_FROM_CAPABILITIES,
            ),
        ):
            with project_fixture() as idempotence_root:
                if idempotence_readme is not None:
                    (idempotence_root / "README.md").write_text(
                        idempotence_readme, encoding="utf-8"
                    )
                idempotence_cap = idempotence_root / "CAPABILITIES.md"
                if idempotence_document is not None:
                    idempotence_cap.write_text(idempotence_document, encoding="utf-8")

                # Tick until the product has reported no change on that many
                # consecutive runs, under a bound well above the two phases any
                # input here can require. The bound keeps a non-converging
                # migration a failed assertion rather than a hung case; the
                # oracle asserts the loop stopped for the first reason and not
                # the second.
                idempotence_ticks: list[tuple[dict, str]] = []
                settled_runs = 0
                while (
                    settled_runs < lumen_reference.MIGRATION_TICKS_PAST_CONVERGENCE
                    and len(idempotence_ticks) < 8
                ):
                    idempotence_envelope = final_json(
                        run_aw(
                            idempotence_root,
                            "capability",
                            "migrate",
                            "--project",
                            "demo",
                        )
                    )
                    idempotence_ticks.append(
                        (
                            idempotence_envelope,
                            idempotence_cap.read_text(encoding="utf-8"),
                        )
                    )
                    settled_runs = (
                        settled_runs + 1
                        if idempotence_envelope.get("changed") is False
                        else 0
                    )
                lumen_reference.assert_migration_is_idempotent_at_its_fixed_point(
                    idempotence_ticks, expected_ticks, subject=subject
                )

        # Surface identity. Every other document here declares each surface
        # once, so the fold that collapses duplicates ran on inputs it could
        # never change, and two of its key's three fields were free.
        with project_fixture() as dedupe_root:
            (dedupe_root / "README.md").write_text(
                lumen_reference.SURFACE_DEDUPE_DOCUMENT, encoding="utf-8"
            )
            dedupe_cap = dedupe_root / "CAPABILITIES.md"
            for _ in range(3):
                final_json(
                    run_aw(dedupe_root, "capability", "migrate", "--project", "demo")
                )
            lumen_reference.assert_surface_identity_is_the_whole_declared_item(
                dedupe_cap.read_text(encoding="utf-8")
            )

        # `(Impl, Verification)` cell pairs folding into a gap status. Four of
        # the five arms are reached only by vocabulary this case never
        # otherwise writes, and each subject owns exactly one row so the
        # capability-level summary attributes to it alone.
        with project_fixture() as gap_status_root:
            (gap_status_root / "README.md").write_text(
                lumen_reference.GAP_STATUS_DOCUMENT, encoding="utf-8"
            )
            gap_status_cap = gap_status_root / "CAPABILITIES.md"
            for _ in range(3):
                final_json(
                    run_aw(gap_status_root, "capability", "migrate", "--project", "demo")
                )
            gap_status_report = final_json(
                run_aw(
                    gap_status_root,
                    "capability",
                    "report",
                    "--project",
                    "demo",
                    "--skip-issue-inventory",
                )
            )
            lumen_reference.assert_work_root_cells_fold_into_gap_status(
                gap_status_report, gap_status_cap.read_text(encoding="utf-8")
            )

        # Gaps with no work roots, which only the YAML reading form produces --
        # the table route derives a work-root row beside every gap, closing the
        # guard above the two status folds for good. Both were free in their
        # entirety, including the order their arms are tried in.
        with project_fixture() as yaml_gap_root:
            (yaml_gap_root / "README.md").write_text(
                lumen_reference.YAML_GAP_DOCUMENT, encoding="utf-8"
            )
            yaml_gap_cap = yaml_gap_root / "CAPABILITIES.md"
            for _ in range(3):
                final_json(
                    run_aw(yaml_gap_root, "capability", "migrate", "--project", "demo")
                )
            lumen_reference.assert_gap_status_renders_its_own_work_root_row(
                yaml_gap_cap.read_text(encoding="utf-8")
            )

        # The three vocabularies that decide what a `key: value` clause means.
        # Every document above writes the handful of spellings its own subject
        # needs, so the rest of each vocabulary -- twenty-two surface keys, the
        # inline EC-dimension split, and all four work-root column enumerations
        # -- was reachable only by writing them.
        contract_fields = _lumen_report(
            root, cap_path, lumen_reference.CONTRACT_FIELD_DOCUMENT
        )
        lumen_reference.assert_surface_keys_are_two_independent_vocabularies(
            contract_fields
        )
        lumen_reference.assert_a_semicolon_without_a_key_stays_in_the_summary(
            contract_fields
        )
        lumen_reference.assert_an_inline_semicolon_splits_ec_dimensions(
            contract_fields
        )
        lumen_reference.assert_an_unrecognized_ec_dimension_is_dropped(
            contract_fields
        )
        lumen_reference.assert_each_out_of_vocabulary_cell_raises_its_own_blocker(
            contract_fields
        )

        # Surfaces and EC dimensions declared as machine tables rather than as
        # contract fields. Every document above uses the field form, so both
        # table parsers -- their alias sets, their defaults, and the row-drop
        # guard whose reachability depends on the table's column shape -- were
        # entirely undriven.
        machine_tables = _lumen_report(
            root, cap_path, lumen_reference.MACHINE_TABLE_DOCUMENT
        )
        lumen_reference.assert_machine_tables_declare_surfaces_and_dimensions(
            machine_tables
        )
        # This document is already canonical, so migration short-circuits and
        # renders nothing. That is asserted as the short-circuit it is -- the
        # envelope plus whole-document byte equality -- because the leg this
        # replaces read the untouched file and reported the absence of a
        # rendered `Surfaces:` field as a product property.
        before_migrate = cap_path.read_text(encoding="utf-8")
        short_circuit = final_json(
            run_aw(root, "capability", "migrate", "--project", "demo")
        )
        lumen_reference.assert_a_canonical_machine_table_document_short_circuits(
            before_migrate, cap_path.read_text(encoding="utf-8"), short_circuit
        )

    # The identical contract through the route migration does rewrite. A
    # README-resident capability relocates, and relocation renders the parsed
    # items as contract fields -- which is where the surface `Verification`
    # cell (#3276) and the table-declared runner (#3278) are lost.
    with project_fixture() as relocation_root:
        (relocation_root / "README.md").write_text(
            lumen_reference.MACHINE_TABLE_DOCUMENT, encoding="utf-8"
        )
        relocated_cap = relocation_root / "CAPABILITIES.md"
        for _ in range(3):
            final_json(
                run_aw(relocation_root, "capability", "migrate", "--project", "demo")
            )
        lumen_reference.assert_relocation_renders_a_machine_table_as_contract_fields(
            relocated_cap.read_text(encoding="utf-8"),
            final_json(
                run_aw(
                    relocation_root,
                    "capability",
                    "report",
                    "--project",
                    "demo",
                    "--skip-issue-inventory",
                )
            ),
        )

    # The efficiency contract fields, which only migration renders.
    with project_fixture() as efficiency_root:
        (efficiency_root / "README.md").write_text(
            lumen_reference.EFFICIENCY_DOCUMENT, encoding="utf-8"
        )
        efficiency_cap = efficiency_root / "CAPABILITIES.md"
        for _ in range(3):
            final_json(
                run_aw(efficiency_root, "capability", "migrate", "--project", "demo")
            )
        lumen_reference.assert_efficiency_fields_render_their_generated_section(
            efficiency_cap.read_text(encoding="utf-8")
        )
        # A leg asserting how the slot merges into an EC dimension was written
        # here, measured, and removed. Neither of its arms was worth a claim: a
        # merge that always pushed is collapsed back by `dedupe_ec_dimensions`
        # and renders the identical report, and a merge that never pushed is
        # already caught by the production document's own rendered dimension
        # list. Reading its report and asserting nothing new would have been
        # scaffolding that looked like coverage.

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
        "contract_fields": contract_fields,
        "machine_tables": machine_tables,
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
                "every declared EC dimension keeps its exact gate command and every Work Root row becomes an exactly named claim and gate, asserted as ordered `(name, command)` pairs over five pairwise-distinct commands -- three dimension runners and two Gate/Evidence cells -- because until round 32 all five were the same string, which bound the dimension names and the gate ids while leaving the command-to-key binding free: a projection emitting one constant runner, or transposing the runners across the dimensions, satisfied both comparisons unchanged",
                "the sweep projection carries the report's capability and claim counts, and its status and verified claim count are pinned to the literals the fixture forces rather than compared against the report's own copy of them, because a scratch project always carries the Python EC and TD inventory blockers and a report with any blocker is `blocked`, so a projection that hardcoded the status passed a self-comparison and the verified claim count compared zero against zero; the branch on which the two legitimately disagree -- current successful verify evidence rewriting the status to `healthy`, the loop status to `done`, and both verified counts to their totals -- needs a fixture carrying real Python EC and TD manifests and committed evidence matching its own git HEAD, and is bound by nothing here",
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
                "the migrated Capability Index and the migrated capability sections of a non-core-first document list the same capabilities in the same core-then-non-core order, so re-parsing the migrated document cannot render a different index again -- which is a precondition for migration converging and not convergence itself, a distinction this label elided for twenty-nine rounds by claiming a fixed point while migrating exactly once, and which is now asserted separately",
                "aw capability migrate erases the tracker state a capability section stored, asserted on the one document carrying both a live Root WI field and a live work-root WI at once -- so the field, the work-root cell, and the gap fallback that root_wi_for_capability reads when the field is blank are each separately observable -- while the same document still derives its split and keeps its gate inventory",
                "aw capability migrate preserves a class the author already declared, even where the derivation from the capability id would have chosen the other class",
                "aw capability migrate relocating a README-resident legacy table into a project with no CAPABILITIES.md preserves each row's tracker state as its Root WI in both the index column and the section field, derives the same core/non-core split, and leaves the README a forwarding pointer instead of the table",
                "aw capability migrate relocating a README whose contract is canonical capability sections preserves each capability's own declared Root WI into both the index column and the section field, which is the branch that resolves it through root_wi_for_capability on live tracker state rather than through the legacy row or through format migration's pre-blanked input",
                "aw capability migrate renders one capability section per capability on that same branch, asserted through both the relocated text and a re-report of it, so a relocation emitting a complete-looking Capability Index over no contract at all cannot pass, with the re-report's capability and claim totals pinned to the counts that exclude the one retired capability the varied-status document declares while its section is still rendered and its id still listed, so the totals cannot be read off the length of the list they are reported beside",
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
                "a relocated capability with no declared Root WI falls back to its first work root's WI in both the index column and the section field, with the second work root's WI never chosen as any capability's Root WI while its own work-root row still carries it, which is the branch of that resolution every Root-WI-declaring input leaves unreachable",
                "aw capability migrate emits the two canonical feature roots exactly when its input classified something, asserted in both directions across relocation shapes that differ in that one property, so neither an unconditional renderer nor one that never emits them can pass",
                "a retired capability is excluded from the verified capability and verified claim counts as well, asserted under --verify where those two accumulators are populated and the classes differ in both, which is the half of the retired filter an unverified report holds vacuously",
                "each legacy row's own Current State, Gaps, and Evidence land in the capability section it becomes, asserted as the whole byte-exact section body per row -- separator included, an earlier revision having compared after stripping trailing newlines on the ground that the blank lines before the next heading were pinned elsewhere, which they were not -- against pairwise-distinct cells rather than as substrings, and bound on both entry points -- format migration, where document-stored tracker state is erased and every Root WI must therefore render `-`, and README relocation, where it is live and each row's own WI must render -- so no field of the rendered section can be a constant one of the two callers happens to agree with",
                "every rendered capability section carries its own Promise, Type, Required Verification, Surfaces, EC Dimensions, Gate Inventory, and Dependencies rather than a shared one, asserted on both re-rendering paths -- format migration and README relocation -- against values made pairwise distinct per capability, down to the surface kind, which is a separate read from the command and summary beside it and stayed constant while the assembled item varied, and down to the EC dimension kind, which is a closed four-value enum that six capabilities cannot be pairwise distinct in and which is therefore held to walking that whole vocabulary instead, the same rule the work-root enum columns are held to, and down to the arity of a surface's own command list, which is itself a loop and was composed at one element by every document, leaving its join separator, its order and its traversal all free until one capability declared two commands in a single item, with the three list-shaped fields read as the exact item list each renders rather than as a block the section contains, because containment pins what a field starts with and nothing after it and appending a duplicate item after either render loop passed on every document including the one that declares two, and with both kinds written in every alias spelling their fold tables accept on a document of their own, because every other document writes only the spelling its table returns, which left each alternative of both tables deletable without changing a rendered byte -- silently for EC dimensions, where an unrecognized kind drops the item rather than carrying it through unfolded -- and with the two surface spellings that are also names of the field itself declared on a shared line rather than one per line, because on their own line they are read as the field name and never reach the fold, and the kind the parse substitutes in their place is the one they would have folded into",
                "the Dependencies field is asserted in both directions, present for the two capabilities that declare one and absent for the four that do not, because no capability declared one at all and the whole block was deletable while the product's own carry-through comment names product dependencies, with one of the two declaring more than one dependency, out of sorted order and with a repeat, and asserted as the whole rendered block, because both declaring capabilities previously carried exactly one and rendering only the first left the loop, the sort, and the deduplication of that parse all rendering the identical document",
                "a capability's Surfaces and Gate Inventory keep every item in declaration order while its EC Dimensions come back in the closed enum's own order, asserted as the exact item list on the one input where a capability declares two of each, because every other document declares exactly one item per list and rendering only the first element of each is byte-identical on those, and with that capability's second dimension declared first, because its two kinds happened to be declared in enum order already, which left the sort and declaration order rendering the identical document and the field labelled as keeping an order it does not keep",
                "the canonical capability section renders its fields in the order the product emits them, asserted as the whole block from `ID:` to the blank line before the work-root table, on two capabilities of the same document -- one declaring two of every list field and no dependency, one declaring one of each and a dependency, so the conditional field that renders last is bound both when it is emitted and when it is not -- because every other assertion on this renderer reads one field at a time and reversing the four conditional blocks left all of them green, and again on the one document that classifies, over three subjects spanning both feature-class values and the absence of one, because a block equality binds the position only of the fields the block contains and `Feature Class` renders in the middle of a block no subject of the two-item document declares",
                "every cell of every work-root row survives relocation, asserted on the one input whose eight rows differ in Kind, Impl, Verification, Maturity, and Gate / Evidence, so none of those five cells can be the constant the other inputs all happen to write",
                "a capability that declares no work-root table still renders a described row, asserted as the whole rendered table per capability on a document where three capabilities declare none -- two keeping a live Root WI, whose row is synthesized from the gap the parse adds for such a capability and carries that tracker state together with a verification cell folded from the capability's own status, one of those two declared verified and the other not so the fold's two answers are distinguishable, and one declaring no tracker state either, whose row is named from the capability's own title and carries the `-` the WI resolution falls back to -- because every other document here gives every capability at least one work root, which is the first of the four blocks that render this table, leaving the three that synthesize a row for an otherwise empty one deletable together without changing a rendered byte; asserted as the whole table rather than as a row it must contain, because those blocks append independently and a condition that stopped excluding the others would leave the right row in place beside a second invented one, and the three capabilities that do declare work roots are held to their authored rows in the same equality; the remaining block, which renders one row per contract claim, is not reachable from a section-shaped README and is not claimed here; and no count stands in for any of this, because a synthesized row is read back as a claim and leaves the report's claim total exactly where the authored tables would have",
                "the Capability Index Maturity column is asserted on the relocation branch as well, against each capability's own Required Verification, because that branch derives it rather than carrying it and the derivation stopped being constant once the fixture varied the field it reads",
                "a promise containing a pipe is escaped into the Capability Index Notes cell it falls back into, asserted through a row reader that splits on unescaped pipes only, so an unescaped pipe adds a column and fails to parse rather than being silently absorbed",
                "the README a section-shaped capability contract was relocated out of keeps only a forwarding pointer and keeps everything that was never part of that contract, asserted on every section-shaped relocation, so relocation can neither leave a second divergent copy of the contract behind nor truncate the README around it",
                "every Capability Index cell a capability arrived with is carried through per capability, asserted across five of the six non-identity columns on an input that differs in every one of those five and in every row -- the sixth, `Root WI`, is uniform `-` across all six rows of that input and is bound by nothing here, because the column the index declares is not the value the product renders into it: with no declared WI the renderer falls back to the first work root, so carry-through of an authored `Root WI` is not observable on this document and is not claimed, which is the branch on which a Production value the product did not derive is reachable -- the derived Production and Maturity values are reachable on the index-less branch too and are asserted there separately",
                "a relocated capability keeps its own Status, the prose prelude above its fields, the prose postlude a different capability of that same document carries below its work-root table -- the other half of a carry-through rule that was asserted on one side only, leaving the whole postlude renderer deletable -- and the `aw ec` efficiency backfill slot two further capabilities carry in that same position, which is not prose at all but is lifted out of it before either prose side is read and re-emitted by its own renderer, so deleting that renderer rendered the identical document until one capability declared a slot; the slot is carried on two capabilities rather than one because the merge that reads it branches on whether the capability already declares an `efficiency` dimension, appending a generated one where it does not and attaching the slot to the declared dimension where it does, and the fixture's own guard used to require the first case and so certified the second unreachable; the appended dimension is now counted once across the whole document, and the capability that already declares an `efficiency` dimension keeps the slot with its dimension list pinned to the single item it authored -- which is deliberately not a claim to have bound the attach arm, because that arm is redundant with the dedupe that follows it and deleting either one alone renders this document byte for byte, so what is bound is that the slot survives on such a capability at all, a claim that fails only when both mechanisms are removed and whose redundancy is reported rather than papered over; and the Impl and Verification columns derived from that status, asserted on the one input that walks the whole six-value status enum, one capability per status, so that every arm of the Verification match is declared rather than the two arms that went undeclared and freely rewritable, with `implemented` derived twice over -- once from the status arm that answers before the gaps are read, and once from a capability that is not verified but whose work roots are all closed, which is the second disjunct of the Impl derivation and was unreachable while every work root of the fixture was closed under a verified status; and, on one of the two capabilities whose status exempts it from the contract requirement -- `candidate` and `retired` are both exempt, four of the six statuses being contract-bearing, and the `retired` arm is a legal second carrier this fixture does not use, the absence of all four optional fields it declared none of -- Type, Surfaces, EC Dimensions, and Required Verification, each guarded separately in the same block and each asserted separately here -- because every capability of every other document declares all four and forcing any one of those emptiness guards true was unobservable, and because an earlier round bound one of the four and left its three siblings free; the fourth of them is the field whose absence is not silent, so the maturity the renderer substitutes in its place is asserted as the fallback literal against a document in which no capability declares that literal itself",
                "the Capability Index header row and its alignment row are asserted as exact literals, because every reader of that table finds its columns by name and would keep passing against a renamed column or a moved right-alignment",
                "the derived Production column is asserted as `not_ready` for all six capabilities of the varied-status relocation, which is the one place `capability_production_summary`\'s own answer is read back -- other legs here read a Production cell too, but the legacy index\'s is emitted by the hardcoded legacy-row branch rather than derived, so it holds against a broken derivation; the derivation runs on every index-less document here -- fourteen six-capability relocation READMEs at module scope alone, not the five an earlier revision of this label claimed -- and its answer was read back by nothing, so the constant `\"ready\"` mutant that this now catches was shipping a production-ready claim on all of them -- that blast radius is the reason the assertion is worth making and not a description of where it is made",
                "the Capability Index is recognized at either heading level the parser accepts, asserted on a `##` index whose columns come through identically to the `###` index every other document here writes, so the level-2 arm of that guard is not free",
                "a Capability Index that declares no Notes column at all falls back to each capability's own promise in that cell, which is the only input that reaches the fallback -- a blank cell does not, because an empty cell is read as `-` -- with the four columns the document still declares carried through unchanged",
                "the Root WI fallback is asserted against every spelling the product treats as an empty table value, including the empty value itself, cycled across the six capabilities, so a fallback that recognizes only the literal `-` leaves the others standing as rendered tracker state",
                "the document relocation creates is asserted as its whole declared frame -- the project title, Brief, the Capabilities heading, and the machine-readable-contract note under it, in that order -- against a project name that appears nowhere in the input, so a frame that dropped a heading, reordered them, or hard-coded the title cannot pass",
                "the forwarding pointer left in the emptied README is asserted as its exact block including the relative link to CAPABILITIES.md, because a pointer that names the contract without linking to it is not a pointer",
                "a README that already carries an authored `## Capability Contract` heading keeps it verbatim and gains no second pointer, which is the early return in the residue renderer that every other input leaves unentered and the reason a second migrate run does not leave a second pointer behind -- one input's contribution to convergence, not a demonstration of it, which is asserted on its own below",
                "all three partial shapes of a contract item round-trip as declared, one per capability of the same input -- a command with no summary, a summary with no command, and an item declaring neither half, which renders as the bare kind alone -- because every other document declares both halves for every item, leaving three of the four arms of each item renderer unentered, and the command-less arms deletable in a way that drops the surface kind and drops the EC dimension name the re-parse needs to read the item back at all, with the neither-half capability declared `candidate` because a bare dimension carries no content and the four contract-bearing statuses require one, `candidate` and `retired` being the two that do not",
                "a capability that declared no gate inventory gets the one its claims imply, asserted as the exact item list of the rendered field across four capabilities of the same document -- one deriving a single gate, one deriving seven refs from two work roots, drawn from both halves of the derivation -- four gates spread unevenly across the two roots, one root carrying three of them and two of those three declared inside a single `;`-separated piece, because a gate id is numbered within one work root, so gates on two different roots are each the first of their own and only a root carrying more than one reaches the numbering at all, while only a piece carrying more than one command reaches the loop nested inside the piece loop, which composed at one command apiece stayed free after the outer loop was bound; three fixtures spread across them with the first root carrying two of its own -- and declared so that the rendered order differs from the declared order, so that joining the list is distinguishable from keeping only its first or only its last element, collecting the claim fixtures before the capability gates from collecting them in work-root order, truncating either half to one ref from rendering it whole, and walking the claims in reverse from walking them in order, because composing the two halves at one ref apiece leaves all four of those truncations rendering the identical document, one whose work-root cell is not backticked and therefore derives through the claim-fixture half of the derivation rather than the claim-gate half, and one declaring only empty-table spellings that gets the single `-` placeholder with its own work-root gate not derived in behind it",
                "a capability with no declared gate inventory and nothing to derive one from renders the placeholder through the derivation's own empty arm, asserted on its own document because the document `aw capability migrate` writes for that input is one `aw capability report` then rejects, named as the exact claim that has neither a gate nor a fixture",
                "format migration supplies the parts of the canonical frame its input is missing, asserted as the whole prefix up to the Capability Index across three inputs missing one part each -- a document with no title, one with no Brief whose lead prose has to be promoted into it, and one with neither, which gets the human-confirmation placeholder instead -- because every other document here arrives with all three parts, so the three repairs were disabled together without changing a rendered byte, and because containment binds that each heading appears somewhere rather than once, in order, and carrying the body it is required to carry, which the author's own prose and the placeholder are indistinguishable under; a fourth input arriving with no Capabilities heading at all is held to the same whole-prefix equality as the other three, which pins the loss of its authored brief prose -- the strip takes the prefix with it when there is no Capabilities heading to stop at and the repair writes the placeholder over it, reported as #3234, and the expected frame names the placeholder where the input authored prose, so fixing the defect fails this leg until the expectation is updated with it; an earlier revision of this label claimed the opposite, that the loss was asserted in neither direction and only the heading insertion bound, which the expectation contradicts, and pinning it is correct here because the brief is one of the frame parts this leg's property is about rather than a loss sitting outside it",
                "a document whose capability contract has not been written yet -- no capability section and no legacy row -- renders neither canonical feature root, because every other input here declares one or the other and the guard for an empty registry was therefore only ever entered with something to render, making it deletable; asserted as byte equality against the whole migrated document rather than as the absence of the two root headings alone, because the interesting failure adds content, which pins in the same string that the legacy level-2 index heading is demoted to the canonical level-3 one and that an index with nothing to list carries a single synthesized row named after the project; the input carries that legacy heading deliberately, because a contract-less document with a canonical frame is answered \"already canonical\" before the renderer is reached, so it leaves the guard unentered and the byte equality holds for a reason unrelated to it",
                "two EC dimension items of the same kind are merged into the one item their halves reconstruct, asserted through the ordinary single-item expectation on a document where two capabilities each declare a summary-only item and a command-only item of one kind -- so a merge that dropped either half renders a shorter item and a merge that did not happen renders two items where one is expected -- with the two capabilities declaring their halves in opposite orders and on different kinds, because the merge fills only the *first* occurrence's empty fields and one order therefore exercises only the runner fill and the other only the summary fill, both of which every other document here left deletable by declaring at most one item per kind",
                "aw capability migrate converges, and the tick that reports no change made none, asserted as the exact ordered list of migrating phases followed by two consecutive no-op runs on both arrival paths -- a README-resident contract, which is relocated and then feature-class migrated, and a resident classified document, which takes the format phase alone -- each arrival path pinned to its own ordered phase list, which is what the second subject adds and the whole of what it adds: the two halves of the conjunction that answers \"already canonical\" turned out to be bound elsewhere already, established by dropping each half in turn and finding the case still failed with this leg neutralized, so this label claims no credit for them; the no-op ticks are pinned to `unchanged`, to `changed: false`, and to the fixed-point sentence naming their own document, which are one observation rather than three -- the product derives `changed` from whether its own stdout starts with `migrated ` and derives `status` from `changed` -- and separately to byte equality against the last migrating tick, which is therefore the only half able to catch a tick that rewrote the document while reporting a no-op, since a `changed` flag re-read from stdout cannot contradict the sentence it was read from; every other leg here migrates exactly once and reads the result, so a migration that rewrote its input on every invocation satisfied all of them and would keep satisfying them while the command never terminated for an adopter driving it to completion; the tick count is itself an observation rather than a constant, the driver ticking until it has seen the no-ops under a bound rather than a fixed number of times, so a migration needing a phase it should not need and one never converging land on different lengths; `kind` is asserted to be the same on the last migrating tick and on the first converged one, because it names the check that ran rather than its outcome, and an assertion reading it as the outcome would bind nothing; and the converged content is deliberately not pinned, because the fixed point preserves the Root WI erasure of #3264 and the escaped-pipe truncation of #3265, and asserting those bytes would hold both losses in place",
                "two declared surfaces are one surface only when their kind, their commands, and their summary all agree, asserted as the exact rendered item list across five capabilities of one document -- an exact duplicate that folds, and three pairs each differing in exactly one of the three key fields, which must not -- because every other document here declares each surface once, so the fold ran only on inputs it could not change and every field of its key was droppable without moving a rendered byte; the kind pair is the one round 31 was missing, and its absence was not visible from the count, because the fourth pair -- an unrecognized kind spelled two ways -- *folds*, which binds the case-fold inside the kind term while leaving the term itself deletable, so a key built from the commands and the summary alone reproduced every expectation this leg made until the fifth capability was added; the rendered item of that unrecognized pair carries the *authored* spelling, which is what the kind normalizer's pass-through fallback produces, but that spelling is bound elsewhere already and this label claims no credit for it -- what the pair adds is that the two spellings are one surface",
                "each `(Impl, Verification)` work-root cell pair reads as one gap status, asserted across five capabilities that own one row apiece so the capability-level summary attributes to that row alone -- the blocked arm through both of its disjuncts, one row blocked on the verification side and one on the implementation side, the `out_of_scope` guard, the `none` spelling of an open row, and the in-progress fallthrough -- because every row this case otherwise *asserts* folds to `closed` except the in-progress fallthrough, which the bad-cell rows of the contract-field document enter as well without anything reading their gap status back -- so four of the five arms are entered nowhere else and the fifth is entered nowhere else that looks -- and because a single blocked row leaves whichever disjunct it does not enter deletable; asserted twice over, once against the gap status named directly by `aw capability report`, which is exact, and once against the Index `Impl` cell it renders through, which is lossy in a way three of the five subjects expose -- both blocked rows and the open row all read `planned` there, so two distinct gap statuses collapse into one cell -- so that binding the internal name alone would leave the fold free to route any status to any cell",
                "the arms of the gap verification fold are tried in the order the product declares them, asserted on a YAML-fenced capability that is itself `verified` and carries the only two gaps whose rendering depends on that order and on nothing else -- a closed gap reads `verified` rather than `passing`, and a blocked gap still reads `blocked` rather than `verified` -- which together pin the blocked arm ahead of the capability-status arm and the capability-status arm ahead of the closed one; this is what an arm-by-arm assertion structurally cannot express, and it is the whole of what this leg adds: the arm *values* of both folds, and the `epic` kind that marks a gap-derived row, are bound already by the capability that declares no work-root table at all, whose synthetic gap enters the same rendering block -- established by mutating each in turn and finding the case still failed with this leg neutralized, so the five gap statuses declared here are declared to reach the two ordering subjects and this label claims no credit for the rest of them",
                "the twenty-two spellings that let an inline `;` open a new surface item are asserted as a set, each against the kind it renders as and paired to that kind through its own command rather than by position, so that the right labels against the wrong keys fails; the two vocabularies are asserted to disagree, fifteen spellings folding onto six canonical labels and seven reaching the report verbatim, which is the shape in which a spelling gets added to the separator test and forgotten in the kind fold; and four look-alike words outside the vocabulary are asserted to open no item at all, so that a splitter consulting no vocabulary produces one item and one splitting on every `; ` produces twenty-seven; all twenty-two sit after a `; ` behind a lead clause whose key is in neither vocabulary, because the leading-key position is parsed before the separator runs and consults no vocabulary at all -- a spelling spent there is bound as a leading key and leaves its separator arm deletable, which is what left `cli` unbound while this label already claimed the set -- and that lead clause doubles as the assertion that the leading-key parse carries an unknown word through as a kind",
                "a `; ` followed by a word outside the surface vocabulary stays inside the summary it was written in, asserted in isolation on a capability declaring a single item so that the clause surviving is the whole observation, rather than only at the tail of a twenty-seven-clause line where a truncation would look the same",
                "EC dimension items split on an inline `;` as well, asserted as two dimensions each keeping its own summary, because every other capability in this case writes one dimension per line and leaves that separator unentered",
                "an EC dimension whose kind is outside the closed enum reaches the report not at all -- and a contract quietly narrower than the one written -- asserted because it was unbound rather than because it is right, in both halves: the item is absent from the dimension list, and no blocker names the capability -- in either of the two spellings the product uses for one, the id its document-level findings use and the title its per-capability validators use, because probing the id alone would certify silence against a rejection phrased the way the validator family beside this one phrases every message it emits -- or the misspelling, which is the half that makes the first mean anything, since a product that dropped the item and rejected the document would be one that told the author and the dimension list alone cannot tell the two apart; the migrated document re-rendering that same dropped line is measured and deliberately left unasserted, so a fix that rejects the misspelling instead of silently dropping it does not have to break this case twice; the drop is separated from the near miss beside it by declaring `security` rather than `behavior`, because an unrecognized kind falling back to `Behavior` is absorbed into a declared behavior item and reads identically to being dropped, and that same choice makes this the only capability in the case that declares dimensions without declaring behavior, which is the only input on which the synthesized runner-less behavior dimension is observable at all",
                "each of the four work-root columns validates its own cell, asserted as four blockers in document order with every message pinned whole because the vocabulary each one names is the assertion\'s content, across four rows that break one column apiece so that a single validator firing four times fails; the capability still parses and still reports its own surface, which is what separates validating every row from abandoning the section at the first bad cell",
                "surfaces and EC dimensions declared as machine tables rather than as contract fields, covering both parsers\' column vocabularies exhaustively -- every spelling of every column of both tables is written by some document here, because `find_table_column` matches on exact normalized equality and a spelling no document writes is an arm deletable without moving a byte, which is what two subjects per parser left true of the third spelling of five columns; the one exception is named rather than quietly skipped: a dimension table headed `Command` cannot be written at all, because `parse_markdown_surface_table` accepts that column too and is tried first, so the table is claimed as a surface table, the dimension is lost and a phantom `CLI` surface appears in its place (#3280), leaving that one arm bound by nothing here and unreachable by any document; also the blank kind cell that reaches the `CLI` default a filled column cannot, and the row-drop guard, whose reachability depends on the table\'s column shape: two capabilities carrying the same three rows disagree about which survive, because the guard requires the command cell empty *and* the summary and verification both blank, and `table_cell` returns the literal `-` for a blank cell of a column the table declares -- so a table declaring either a summary or a verification column can never satisfy it, which is not expressible in one table and needs the pair; the table route\'s runner is asserted with its backticks intact against the field route\'s stripped spelling, because the two routes genuinely disagree and asserting one alone would leave a refactor free to unify them in either direction",
                "a capability document already in canonical form short-circuits migration -- the envelope's `unchanged` and `changed: false`, which are one observation rather than two because the product derives `changed` from whether its own stdout starts with `migrated ` and derives `status` from `changed`, plus the file being byte-identical, which is the independent half and the only one that can catch a run that rewrote the document while reporting a no-op -- asserted as whole-document equality rather than per-line containment, and labelled as the short-circuit it is: this leg claims nothing about what migration renders, because the leg it replaces read the same untouched file and reported the absence of a rendered `Surfaces:` field as a product property, which was a property of its own input satisfied by a command that did nothing",
                "relocating the identical machine-table contract out of `README.md` rewrites all eight tables as `Surfaces:` and `EC Dimensions:` items, asserted as each block\'s exact ordered line list, with each subject\'s own table rows asserted absent by the rows it wrote rather than by two hardcoded header spellings that matched five of the eight, and cross-checked against the re-parsed report, which observes the migrated document through a second product surface rather than repeating the text comparison and is the behaviour the leg above was previously claimed to have refuted; two contract fields do not survive that round trip and are pinned as they render with their issue named, so a fix fails this leg by design rather than silently widening it -- which is the opposite of what this case does for #3264, #3265, #3272 and #3274, and the difference is that those four sit outside the property their leg claims, where pinning would hold a loss in place inside an assertion that is not about it, while these two sit inside a block this leg asserts exactly and there is no way to assert a block exactly while excusing two of its lines, the weaker alternative being the shape that produced the vacuous leg this one replaces -- every relocated surface has lost its `Verification` cell because `render_surface_field_items` renders three of four fields, four such cells across three capabilities having declared one (#3276), and the three capabilities whose dimensions were tables lose their runner to doubled backticks while the five whose dimensions were contract fields keep theirs (#3278), the contrast being what attributes the loss to the table route rather than to relocation at large; the out-of-enum dimension is absent from the relocated document too, so the drop that merely narrows the contract on the field route (#3274) is destructive on this one",
                "the `Efficiency Operating Point:` and `Efficiency Cube:` contract fields render into their own generated section, asserted as the ordered sequence of blocks across three capabilities -- both halves, each half alone -- because a capability declaring both cannot show what a missing half renders as and the two missing-half spellings are separate literals a renderer could produce for only one of them; each block is compared as its own exact ordered line list rather than as a mapping of its colon-bearing lines, because a mapping binds neither the order of the two fields nor the blank lines framing them and drops any line the renderer added without a colon; read positionally rather than as a field of the capability that produced it, because the section is emitted after that capability\'s work-root table and its own heading closes the block it belongs to, which is #3272 and is why the round trip is deliberately not asserted here",
                "Lumen's production capability contract is byte-identical before and after the fixture run -- a property of this external contract rather than of `aw capability`, listed so a reader can see the fixture is barred from mutating a real project's contract, and counting toward nothing the product promises",
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

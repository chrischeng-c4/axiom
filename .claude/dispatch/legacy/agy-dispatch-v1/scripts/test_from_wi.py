#!/usr/bin/env python3
"""The projection is measured by what the round documents it emits say.

Not by whether it parsed. A projection that reads every section and then writes
a write allowlist missing one change point produces a round whose worker is
denied halfway through, and the parse was green the whole time. So the rows here
assert the emitted document, and the central one asserts that the pair lints:
a projection whose output `lint` refuses is worth less than no projection, since
the author now has to find what went wrong in generated prose.
"""
from __future__ import annotations

import re
import tempfile
import textwrap
import unittest
from pathlib import Path

import agy_dispatch
import from_wi

FILL_COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)

BODY = """\
## Goal

Two bodies differing only inside an AW-owned marker block produce different
stored digests today; they must produce the same one.

## How

### Verified premises

- `src/thing.rs:2` hashes the whole body, markers included.
- `src/thing.rs:1` takes the body by reference, so the reducer owns no copy.

### Change points

- `src/thing.rs` - the reducer that stores the digest.
- `src/digest.rs` - the helper it calls.

### Frozen decisions

- The marker vocabulary itself is settled.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `cargo test -p target --lib some_gate` | no test compares two marker bodies | a test asserts the two digests agree | the assertion names both bodies, which nothing else constructs |

### Negative control

Restore `strip_aw_marker_blocks` to a no-op; the gate must FAIL. Restore to
sha256 `abc123def4567890`.

## Never

You are implementing this change; you are not redesigning the marker vocabulary.

### Must not touch

- `src/markers.rs`

### Must not do

- Do not introduce a second notion of what AW owns.
"""


class ProjectionTest(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(
            lambda: __import__("shutil").rmtree(self.root, ignore_errors=True)
        )
        (self.root / "src").mkdir()
        # The premises cite lines 1 and 2 of this file, and the projection reads
        # them rather than trusting the work item to have quoted them right.
        (self.root / "src" / "thing.rs").write_text(
            "fn store(body: &str) {\n    let digest = canonical_digest(body);\n}\n"
        )
        (self.root / "src" / "digest.rs").write_text("pub fn canonical_digest() {}\n")
        (self.root / "src" / "markers.rs").write_text('pub const M: &str = "AW";\n')
        self.fields = from_wi.project(BODY, self.root)

    def profile(self) -> dict:
        # `oracle_findings` cross-checks the oracle's `## Scope` table against
        # the profile's write scope, so a stub carrying no scope keys reads as
        # a profile that authorizes nothing and every scope row is a mismatch.
        # Derived from the same `fields["writes"]` the oracle table is rendered
        # from, because that is what the real pipeline does: `make_profile.py`
        # takes those paths as `--write` and emits all three keys together.
        # A hand-copied list here would pass while the projection drifted.
        return {
            "root": str(self.root),
            "task_contract": {
                "gate_command": "cargo test -p target --lib some_gate"
            },
            "task_commands": {
                "allow": ["cargo test -p target --lib some_gate"]
            },
            # No budget and no range, matching the `| path | none |` rows
            # `render_oracle` writes when the change points carry neither.
            "allowed_repo_writes": list(self.fields["writes"]),
            "path_change_budgets": {},
            "path_line_ranges": {},
        }

    # -- what the round is bounded by -------------------------------------

    def test_the_write_allowlist_is_the_change_points(self) -> None:
        """The transcription this script exists to remove.

        A write allowlist retyped from the change points is the one that
        quietly stops matching them, and the worker learns about the mismatch
        as a denial partway through a round already paid for.
        """
        self.assertEqual(
            self.fields["writes"], ["src/thing.rs", "src/digest.rs"]
        )

    def test_the_gate_is_the_acceptance_command_verbatim(self) -> None:
        """Verbatim, from inside the backticks.

        The gate is compared byte-for-byte against `task_commands.allow` and
        against the oracle's own `## Gate` fence, so a command carrying its
        surrounding prose is a round that cannot run the thing it is judged by.
        """
        self.assertEqual(
            self.fields["gates"], ["cargo test -p target --lib some_gate"]
        )

    def test_must_not_touch_becomes_the_design_input(self) -> None:
        """`make_profile.py` refuses a bounded-write round with no design input.

        The obvious source -- references outside the write set -- is routinely
        empty, because a GHAN body grounds its premises in the files it is about
        to change. Here both premises name `src/thing.rs`, which is also a
        change point, so without the must-not-touch fallback this round could
        not produce a profile at all.
        """
        self.assertEqual(self.fields["design_inputs"], ["src/markers.rs"])
        argv = from_wi.profile_argv(self.fields, "1234", ["src"])
        self.assertIn("--design-input", argv)
        self.assertEqual(argv[argv.index("--design-input") + 1], "src/markers.rs")

    def test_a_must_not_touch_entry_that_is_not_a_file_is_not_frozen(self) -> None:
        """A design input has to be readable to be frozen.

        `### Must not touch` legitimately carries directories and prose limits;
        handing one to `--design-input` as though it were a file produces a
        profile whose frozen artifact cannot be hashed.
        """
        fields = from_wi.project(
            BODY.replace("- `src/markers.rs`", "- `src/nowhere.rs`"), self.root
        )
        self.assertEqual(fields["design_inputs"], [])

    def test_the_expected_observation_is_the_target_not_the_current(self) -> None:
        """Column 3, not column 2.

        Off by one and the oracle expects what the tree already does, so the
        round is judged green before the worker starts and `prove` has no pair
        to separate. Both cells are prose, so nothing downstream can tell them
        apart.
        """
        command, expected, why = self.fields["measurements"][0]
        self.assertEqual(expected, "a test asserts the two digests agree")
        self.assertNotIn("no test compares", expected)
        self.assertIn("nothing else constructs", why)

    def test_the_scope_is_the_project_not_its_parent(self) -> None:
        """Two segments, for the layout the repository actually has.

        One segment freezes every project under `apps/` to bound a change in
        one of them, and the whole path's parent leaves a sibling directory the
        work item never named writable. The fixture above is too shallow to
        tell those apart, so this row uses a real repository path.
        """
        self.assertEqual(
            from_wi.derived_scope(
                ["apps/agentic-workflow/src/issues/ghan.rs"]
            ),
            ["apps/agentic-workflow"],
        )

    def test_the_scope_covers_every_change_point(self) -> None:
        """`make_profile.py` freezes the complement *within* each scope.

        A scope that does not contain a change point leaves that point neither
        frozen nor writable, which reads as an ordinary write at review and as
        a scope finding at verify.
        """
        scope = from_wi.derived_scope(self.fields["writes"])
        for path in self.fields["writes"]:
            self.assertTrue(
                any(path.startswith(f"{entry}/") for entry in scope),
                f"{path} is under no derived scope {scope}",
            )

    # -- what the documents say -------------------------------------------

    def test_current_behavior_quotes_the_checkout_not_the_work_item(self) -> None:
        """`lint` checks the quote against the tree, so the tree is the source.

        A hand-copied quote is a finding waiting for the base to move. Here the
        work item says nothing about what line 2 contains, and the quote still
        carries it.
        """
        injection = from_wi.render_injection(self.fields)
        self.assertIn("let digest = canonical_digest(body);", injection)
        self.assertNotIn("let digest", BODY)

    def test_a_premise_coordinate_past_the_file_is_not_quoted(self) -> None:
        """A stale coordinate produces no quote rather than a wrong one.

        Quoting whatever happens to sit at that offset would hand the worker a
        line the work item never meant, presented as the code as it stands.
        """
        stale = BODY.replace("`src/thing.rs:2`", "`src/thing.rs:900`")
        fields = from_wi.project(stale, self.root)
        self.assertEqual(len(fields["quotes"]), 1)
        self.assertNotIn("canonical_digest(body)", from_wi.render_injection(fields))

    def test_every_premise_about_one_file_reaches_its_reference_row(self) -> None:
        """One row per file, as the contract asks -- but not one premise per file.

        Keeping only the first would drop the reason the second premise was
        written down, and a reason cell that lost half its content is how
        "relevant context" gets back in.
        """
        self.assertEqual(len(self.fields["references"]), 1)
        reason = dict(self.fields["references"])["src/thing.rs"]
        self.assertIn("hashes the whole body", reason)
        self.assertIn("takes the body by reference", reason)

    def test_out_of_scope_carries_both_never_lists(self) -> None:
        """Must-not-touch bounds where, must-not-do bounds what.

        The write allowlist already blocks the first mechanically; dropping
        either list here leaves the worker free to redesign on the way past.
        """
        self.assertIn("src/markers.rs", " ".join(self.fields["out_of_scope"]))
        self.assertIn(
            "second notion of what AW owns", " ".join(self.fields["out_of_scope"])
        )

    def test_the_negative_control_becomes_a_measurement_row(self) -> None:
        """A control named only in prose is a sentence about a control.

        The oracle contract reads it that way too: the table would lint green
        while measuring nothing.
        """
        oracle = from_wi.render_oracle(self.fields)
        rows = [
            line for line in oracle.splitlines()
            if line.startswith("|") and "negative control" in line
        ]
        self.assertEqual(len(rows), 1, oracle)
        self.assertIn("must FAIL", rows[0])
        self.assertIn("abc123def4567890", rows[0])

    # -- the acceptance ----------------------------------------------------

    def test_exactly_the_unsourced_slots_are_left_as_forms(self) -> None:
        """Four slots, named, and no others.

        The list is the script's claim about what a work item cannot say. If a
        fifth slot silently joined it the projection would still look like it
        worked, and the author would find the gap at `lint` with no idea it was
        meant to be derived.
        """
        pair = from_wi.render_oracle(self.fields) + from_wi.render_injection(
            self.fields
        )
        unfilled = {
            heading
            for heading, body in _sections(pair).items()
            if FILL_COMMENT.search(body)
        }
        self.assertEqual(
            unfilled,
            {
                "## Fabrication tells",
                "## Required change",
                "## Shape to follow",
                "## Definition of done",
            },
        )

    def test_the_filled_projection_lints_clean(self) -> None:
        """The row the rest of them exist to support.

        Every derived slot has to satisfy the same structural contract a
        hand-authored round does. A projection whose output `lint` refuses is
        worth less than none, because the author now debugs generated prose.
        """
        oracle, injection = _fill(
            from_wi.render_oracle(self.fields),
            from_wi.render_injection(self.fields),
        )
        profile = self.profile()
        self.assertEqual(agy_dispatch.oracle_findings(profile, oracle), [])
        self.assertEqual(
            agy_dispatch.injection_findings(profile, injection, oracle, {}), []
        )


class RefusalTest(unittest.TestCase):
    """A half-projected round is worse than none.

    Each of these is a work item that would otherwise emit a document somebody
    then corrects by hand -- which is the transcription the script removes.
    """

    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(
            lambda: __import__("shutil").rmtree(self.root, ignore_errors=True)
        )
        (self.root / "src").mkdir()
        (self.root / "src" / "thing.rs").write_text("fn store() {}\n")

    def refuses(self, body: str, fragment: str) -> None:
        with self.assertRaises(SystemExit) as caught:
            from_wi.project(body, self.root)
        self.assertIn(fragment, str(caught.exception))

    def test_a_legacy_body_is_refused_by_section_name(self) -> None:
        self.refuses(
            "## Problem\n\nThe digest moves.\n", "no `## Goal` section"
        )

    def test_an_empty_goal_is_refused(self) -> None:
        """An empty claim projects into an oracle that claims nothing.

        `lint` would take it: `## Claim` exists and the table below it is
        populated, so the round dispatches with nothing to be wrong about.
        """
        self.refuses(
            BODY.replace(
                "Two bodies differing only inside an AW-owned marker block "
                "produce different\nstored digests today; they must produce "
                "the same one.\n",
                "",
            ),
            "`## Goal` is empty",
        )

    def test_a_change_point_with_no_path_is_refused(self) -> None:
        self.refuses(
            BODY.replace("- `src/digest.rs` - the helper it calls.",
                         "- the helper it calls"),
            "write allowlist",
        )

    def test_an_unbackticked_gate_command_is_refused(self) -> None:
        self.refuses(
            BODY.replace("`cargo test -p target --lib some_gate`",
                         "cargo test -p target --lib some_gate"),
            "not backticked",
        )

    def test_a_missing_negative_control_is_refused(self) -> None:
        self.refuses(
            BODY.replace("### Negative control", "### Notes"),
            "Negative control",
        )

    def test_an_empty_never_is_refused(self) -> None:
        self.refuses(
            BODY.replace("- `src/markers.rs`\n", "").replace(
                "- Do not introduce a second notion of what AW owns.\n", ""
            ),
            "Out of scope",
        )

    def test_a_gate_table_with_four_columns_is_refused(self) -> None:
        """The column set is the projection's contract with the table.

        Reading `target` out of column 3 when the table has four columns puts
        the *current* observation in the oracle as the expected one, and the
        round then measures the defect it was dispatched to remove.
        """
        self.refuses(
            BODY.replace(
                "| 1 | `cargo test -p target --lib some_gate` | no test compares "
                "two marker bodies | a test asserts the two digests agree | the "
                "assertion names both bodies, which nothing else constructs |",
                "| 1 | `cargo test -p target --lib some_gate` | a test asserts "
                "the two digests agree | the assertion names both bodies |",
            ),
            "columns",
        )


def _sections(text: str) -> dict[str, str]:
    return from_wi.sections(text, 2)


def _fill(oracle: str, injection: str) -> tuple[str, str]:
    """Author the four unsourced slots, as the controller would."""
    oracle = FILL_COMMENT.sub("", oracle).replace(
        "## Fabrication tells\n\n\n-\n",
        "## Fabrication tells\n\n- a gate green because no fixture body carries "
        "a marker block at all\n",
    )
    injection = FILL_COMMENT.sub("", injection)
    injection = injection.replace(
        "## Required change\n\n\n-\n",
        "## Required change\n\n- Two bodies differing only inside a marker "
        "block store the same digest.\n",
    )
    injection = injection.replace(
        "## Shape to follow\n\n\n",
        "## Shape to follow\n\n`canonical_digest` already takes the body by "
        "reference; match it.\n\n",
    )
    injection = injection.replace(
        "## Definition of done\n\n\n",
        "## Definition of done\n\nThe new check joins the existing `mod tests` "
        "in `src/thing.rs`.\n\n",
    )
    return oracle, injection


if __name__ == "__main__":
    unittest.main()

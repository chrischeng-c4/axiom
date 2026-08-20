#!/usr/bin/env python3
"""Prove each META-doc rule fires, and fires only on its own defect.

`meta.py` reports 103 findings against this checkout, and a number that large is
exactly the shape a broken detector produces. So every rule here is measured
against a fixture where the defect and its absence sit side by side: `rotten/`
carries all four defects, `clean/` carries none, and a rule that fires on both
is a rule measuring nothing. Both projects are in the same run, so the two
answers come from one invocation rather than from two that could have differed
for some other reason.

The fixture is a throwaway `tempfile` git repository with its own index. That is
not tidiness -- `meta.py` derives its population from `git ls-files`, so a
fixture that is not a git repository would exercise the error path instead of
the rule, and one built inside this checkout would be measured by the real run
as well.

Four defects here were found by running the rules rather than by reasoning
about them, and each has a row below:

  * `_Box[int](42)` in the CPython seed corpus is markdown link syntax. M3 read
    two of those as broken links until it learned to skip fenced blocks.
  * `apps/jet/README.md:63-64` is a ```bash block with two bare `aw` lines. The
    backtick-anchored detector could not see the most copy-and-run shape in the
    repository, which is why M2 reads a fence too -- the opposite direction from
    M3, from the same fence walker.
  * A `--path` that matches nothing used to report CLEAN. A mistyped path
    certifying the repository is the false green this suite exists to refuse, so
    it exits 2 now, and 2 is asserted distinct from 1 below.
  * An *inline* code span is the same exemption a fence is, and M3 did not know
    it. The document that exposed this is `verification/README.md`, which quotes
    the two rows above: writing down what M3 must not report made M3 report it.
    The live count did not move, so the fixture rows are the whole measurement.
"""
import json
import pathlib
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import (META_SCRIPT, PLUGIN_DIR, REPO,  # noqa: E402
                    load_script_module, pinned_interpreter)

GIT = ("git", "-c", "core.fsmonitor=false")

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


# --------------------------------------------------------------------------
# the fixture
# --------------------------------------------------------------------------
# Line numbers are asserted, so the content is authored as lines and the
# comments are the line numbers. A rule that reports the wrong coordinate sends
# a reader to the wrong place, which is a defect a count cannot see.
ROOT_README = [
    "# Fixture",                                                    # 1
    "",                                                             # 2
    "An inventory, not a promise. No Brief heading here on purpose.",  # 3
]
ROOT_CONTRIBUTING = [
    "# Contributing",                                               # 1
    "",                                                             # 2
    "Nothing here.",                                                # 3
]
CLEAN_README = [
    "# Clean",                                                      # 1
    "",                                                             # 2
    "## Brief",                                                     # 3
    "",                                                             # 4
    "A project whose META-docs are intact.",                        # 5
    "",                                                             # 6
    "## Capabilities",                                              # 7
    "",                                                             # 8
    "- Nothing claimed yet.",                                       # 9
    "",                                                             # 10
    "See [CONTRIBUTING.md](CONTRIBUTING.md).",                      # 11
    "",                                                             # 12
    # Three gate shapes M5 and M6 must both accept. The first names its target,
    # the second names the lib, and the third proves `--features` is understood
    # as taking a value -- read as a filter, `extra` would be an M5 on a
    # correctly written gate, which is the false positive that would make the
    # rule unusable.
    "- Gate: `cargo test -p cleanpkg --test real_target`",          # 13
    "- Gate: `cargo test -p cleanpkg --lib`",                       # 14
    "- Gate: `cargo test -p cleanpkg --features extra --test real_target`",  # 15
    "",                                                             # 16
    # The two truncations, each a false positive this rule reported before the
    # span learned to stop. Everything after `#` is a comment, and the words
    # inside the JSON string belong to the string.
    "```bash",                                                      # 17
    "cargo test -p cleanpkg   # spec/compare units + stub-server",   # 18
    "```",                                                          # 19
    "",                                                             # 20
    'Quoted: "cargo test -p cleanpkg", as JSON would carry it.',     # 21
]
# Not a META-doc, so it is staged apart from `TRACKED` -- the scanned-count row
# asserts `len(TRACKED)`, and a manifest counted there would make the population
# disagree with itself. M6 resolves `-p cleanpkg` and `--test real_target`
# against this file and nothing else.
CLEAN_MANIFEST = [
    "[package]",                                                    # 1
    'name = "cleanpkg"',                                            # 2
    'version = "0.0.0"',                                            # 3
    "autotests = false",                                            # 4
    "",                                                             # 5
    "[[test]]",                                                     # 6
    'name = "real_target"',                                         # 7
    'path = "e2e/real_target.rs"',                                  # 8
]
CLEAN_CONTRIBUTING = [
    "# Contributing to Clean",                                      # 1
    "",                                                             # 2
    "Run the gate.",                                                # 3
]
ROTTEN_README = [
    "# Rotten",                                                     # 1
    "",                                                             # 2
    "## Capabilities",                                              # 3
    "",                                                             # 4
    "- Nothing claimed yet.",                                       # 5
    "",                                                             # 6
    "Run `aw capability check --project rotten` to verify.",        # 7
    "",                                                             # 8
    "```bash",                                                      # 9
    "aw health --project rotten",                                   # 10
    "```",                                                          # 11
    "",                                                             # 12
    "### A Capability",                                             # 13
    "",                                                             # 14
    "Status: verified",                                             # 15
    "",                                                             # 16
    "| Capability | Maturity | Production |",                       # 17
    "|---|---|---|",                                                # 18
    "| A | smoke | ready |",                                        # 19
    "",                                                             # 20
    "- Gate: `cargo test -p cleanpkg some_filter`",                 # 21
    "- Gate: `cargo test -p cleanpkg --test gone_target`",          # 22
    "- Gate: `cargo test -p nosuchpkg --lib`",                      # 23
]
ROTTEN_CONTRIBUTING = [
    "# Contributing to Rotten",                                     # 1
    "",                                                             # 2
    "<!-- aw:meta:project-contributing:start -->",                  # 3
    "## Contributing",                                              # 4
    "<!-- aw:meta:project-contributing:end -->",                    # 5
    "",                                                             # 6
    "See [the design](docs/gone.md) and [the sibling](README.md).",  # 7
    "",                                                             # 8
    "<!-- aw:meta:never-closed:start -->",                          # 9
    "",                                                             # 10
    "```python",                                                    # 11
    "_b = _Box[int](42)",                                           # 12
    "```",                                                          # 13
    "",                                                             # 14
    "<!-- aw:meta:never-opened:end -->",                            # 15
    "",                                                             # 16
    # The next three carry the *same* link as line 7 -- same text, same
    # missing target -- so the only difference between the reported one and
    # the unreported ones is the backticks around them.
    "A span like `[the design](docs/gone.md)` is a literal.",       # 17
    "",                                                             # 18
    "And ``[the design](docs/gone.md)`` in a longer run.",          # 19
    "",                                                             # 20
    "But `code` beside [the design](docs/gone.md) is a link.",      # 21
]
DOCSONLY_README = [
    "# Docs Only",                                                  # 1
    "",                                                             # 2
    "No CONTRIBUTING.md beside this, so it is not a project.",      # 3
]
# Written and never staged. Everything wrong with it must go unreported.
UNTRACKED_README = [
    "# Untracked",                                                  # 1
    "",                                                             # 2
    "<!-- aw:meta:project-readme:start -->",                        # 3
    "Run `aw wi list` and read [nothing](nowhere.md).",             # 4
    "<!-- aw:meta:project-readme:end -->",                          # 5
]

TRACKED = {
    "README.md": ROOT_README,
    "CONTRIBUTING.md": ROOT_CONTRIBUTING,
    "clean/README.md": CLEAN_README,
    "clean/CONTRIBUTING.md": CLEAN_CONTRIBUTING,
    "rotten/README.md": ROTTEN_README,
    "rotten/CONTRIBUTING.md": ROTTEN_CONTRIBUTING,
    "docsonly/README.md": DOCSONLY_README,
}
UNTRACKED = {
    "untracked/README.md": UNTRACKED_README,
    "untracked/CONTRIBUTING.md": ROOT_CONTRIBUTING,
}
# Tracked, and deliberately outside `TRACKED`: it is staged so M6 can resolve a
# package against it, and it is not a META-doc so it must not reach the
# population the scanned-count row checks.
TRACKED_AUX = {
    "clean/Cargo.toml": CLEAN_MANIFEST,
}

LAUNCH = pinned_interpreter()


def write(root, files):
    for rel, lines in files.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run(root, *args):
    return subprocess.run([*LAUNCH, str(META_SCRIPT), "check", "--repo", str(root), *args],
                          capture_output=True, text=True)


def report(root, *args):
    """(exit code, parsed JSON) for one run; JSON is None when it did not run."""
    proc = run(root, "--format", "json", *args)
    try:
        return proc.returncode, json.loads(proc.stdout)
    except json.JSONDecodeError:
        return proc.returncode, None


def rows(data, rule=None, path=None):
    return [(f["path"], f["line"]) for f in data["findings"]
            if (rule is None or f["rule"] == rule)
            and (path is None or f["path"] == path)]


with tempfile.TemporaryDirectory(prefix="meta-flow-") as tmp:
    root = pathlib.Path(tmp) / "checkout"
    root.mkdir()
    write(root, TRACKED)
    write(root, TRACKED_AUX)
    write(root, UNTRACKED)
    subprocess.run([*GIT, "init", "-q"], cwd=root, capture_output=True, text=True)
    staged = subprocess.run([*GIT, "add", "--", *TRACKED, *TRACKED_AUX], cwd=root,
                            capture_output=True, text=True)
    check("the fixture staged its tracked files", staged.returncode == 0,
          staged.stderr.strip())

    code, data = report(root)
    check("the fixture run produced parseable JSON", data is not None,
          "" if data else run(root, "--format", "json").stderr.strip()[:200])
    if data is None:
        print("\n=> RED: the fixture never ran; nothing below was measured")
        sys.exit(1)

    # -- population -------------------------------------------------------
    check("only tracked META-docs are scanned",
          data["population"]["scanned"] == len(TRACKED),
          f"scanned={data['population']['scanned']} expected={len(TRACKED)}")
    check("an unstaged META-doc is invisible to every rule",
          not [f for f in data["findings"] if f["path"].startswith("untracked/")],
          f"{[f['path'] for f in data['findings'] if f['path'].startswith('untracked/')]}")
    check("a directory with a README and no CONTRIBUTING is not a project",
          data["population"]["project_readmes"] == 2,
          f"project_readmes={data['population']['project_readmes']}; "
          "clean/ and rotten/ only")

    # -- M1 ---------------------------------------------------------------
    check("M1 reports an orphaned marker pair at its start line",
          ("rotten/CONTRIBUTING.md", 3) in rows(data, "M1"),
          f"{rows(data, 'M1')}")
    check("M1 reports a start that is never closed",
          ("rotten/CONTRIBUTING.md", 9) in rows(data, "M1"), f"{rows(data, 'M1')}")
    check("M1 reports an end that never opened",
          ("rotten/CONTRIBUTING.md", 15) in rows(data, "M1"), f"{rows(data, 'M1')}")
    check("M1 fires on nothing else in the fixture",
          len(rows(data, "M1")) == 3, f"{rows(data, 'M1')}")

    # -- M2 ---------------------------------------------------------------
    check("M2 reports a backticked command in prose",
          ("rotten/README.md", 7) in rows(data, "M2"), f"{rows(data, 'M2')}")
    check("M2 reports a bare command inside a fenced block",
          ("rotten/README.md", 10) in rows(data, "M2"), f"{rows(data, 'M2')}")
    check("M2 fires on nothing else in the fixture",
          len(rows(data, "M2")) == 2, f"{rows(data, 'M2')}")
    # The shipped exemptions quote sentences in *this* checkout. Run against
    # the fixture they matched nothing, and the first run of this gate reported
    # four stale-exemption findings about the script's own table. The rule is
    # still measured -- by the unit rows below, and live against `REPO`.
    check("the exemption table is not audited against a checkout it is not about",
          not [f for f in data["findings"] if f["line"] == 0],
          f"{[(f['path'], f['message']) for f in data['findings'] if f['line'] == 0]}")

    # -- M3 ---------------------------------------------------------------
    check("M3 reports a relative link whose target is absent",
          rows(data, "M3") == [("rotten/CONTRIBUTING.md", 7),
                               ("rotten/CONTRIBUTING.md", 21)], f"{rows(data, 'M3')}")
    # The same line carries `[the sibling](README.md)`, which resolves. A rule
    # that reported the line rather than the target would pass the row above
    # while being unable to tell the two links apart.
    check("M3 names the broken target and not the resolving one beside it",
          all("docs/gone.md" in f["message"] and "README.md" not in f["message"]
              for f in data["findings"] if f["rule"] == "M3"),
          f"{[f['message'] for f in data['findings'] if f['rule'] == 'M3']}")
    check("M3 does not read `_Box[int](42)` in a fence as a link",
          ("rotten/CONTRIBUTING.md", 12) not in rows(data, "M3"), f"{rows(data, 'M3')}")
    # Lines 17, 19 and 21 are the whole control between them: the same broken
    # link three times, suppressed twice by the backticks around it and
    # reported once when the backticks are around something else. Asserting
    # only the suppression would also pass if the span blanked the whole line.
    check("M3 does not read a link inside an inline code span as a link",
          ("rotten/CONTRIBUTING.md", 17) not in rows(data, "M3"), f"{rows(data, 'M3')}")
    check("an inline span is closed by a run of its own length, not by one tick",
          ("rotten/CONTRIBUTING.md", 19) not in rows(data, "M3"), f"{rows(data, 'M3')}")
    check("a span elsewhere on the line does not suppress the link beside it",
          ("rotten/CONTRIBUTING.md", 21) in rows(data, "M3"), f"{rows(data, 'M3')}")

    # -- M4 ---------------------------------------------------------------
    check("M4 reports a project README with no `## Brief`",
          rows(data, "M4") == [("rotten/README.md", 1)], f"{rows(data, 'M4')}")
    check("M4 names the section that is missing",
          all("## Brief" in f["message"] for f in data["findings"] if f["rule"] == "M4"),
          f"{[f['message'] for f in data['findings'] if f['rule'] == 'M4']}")
    check("M4 exempts the repository root, whose README is an inventory",
          not rows(data, "M4", "README.md"), f"{rows(data, 'M4')}")
    check("M4 does not reach a README with no CONTRIBUTING beside it",
          not rows(data, "M4", "docsonly/README.md"), f"{rows(data, 'M4')}")

    # -- M5 ---------------------------------------------------------------
    check("M5 reports a bare test-name filter",
          rows(data, "M5") == [("rotten/README.md", 21)], f"{rows(data, 'M5')}")
    check("M5 names the filter word it refuses",
          all("some_filter" in f["message"]
              for f in data["findings"] if f["rule"] == "M5"),
          f"{[f['message'] for f in data['findings'] if f['rule'] == 'M5']}")
    check("M5 does not read the value of `--test` as a filter",
          not rows(data, "M5", "rotten/README.md") or
          ("rotten/README.md", 22) not in rows(data, "M5"), f"{rows(data, 'M5')}")

    # -- M6 ---------------------------------------------------------------
    check("M6 reports a `--test` target the named package does not declare",
          ("rotten/README.md", 22) in rows(data, "M6"), f"{rows(data, 'M6')}")
    check("M6 reports a package that is not in the checkout",
          ("rotten/README.md", 23) in rows(data, "M6"), f"{rows(data, 'M6')}")
    check("M6 tells an absent target from an absent package",
          any("--test gone_target" in f["message"] for f in data["findings"]
              if f["rule"] == "M6")
          and any("-p nosuchpkg" in f["message"] for f in data["findings"]
                  if f["rule"] == "M6"),
          f"{[f['message'] for f in data['findings'] if f['rule'] == 'M6']}")
    check("M6 fires on nothing else in the fixture",
          len(rows(data, "M6")) == 2, f"{rows(data, 'M6')}")

    # -- M7 ---------------------------------------------------------------
    check("M7 reports a self-graded field",
          ("rotten/README.md", 15) in rows(data, "M7"), f"{rows(data, 'M7')}")
    check("M7 reports a Capability Index table header",
          ("rotten/README.md", 17) in rows(data, "M7"), f"{rows(data, 'M7')}")
    check("M7 fires on nothing else in the fixture",
          len(rows(data, "M7")) == 2, f"{rows(data, 'M7')}")

    # -- M5/M6/M7 reach project READMEs and nothing else -------------------
    # `rotten/CONTRIBUTING.md` is in the same run and carries no capability
    # rules by design. Without this row, a rule scoped to every META-doc would
    # pass every assertion above.
    check("M5, M6, and M7 do not reach a CONTRIBUTING.md",
          not [f for f in data["findings"]
               if f["rule"] in ("M5", "M6", "M7")
               and f["path"].endswith("CONTRIBUTING.md")],
          f"{[(f['rule'], f['path']) for f in data['findings'] if f['rule'] in ('M5', 'M6', 'M7')]}")

    # -- the negative control every rule shares ---------------------------
    # `clean/` is in the same run as `rotten/`, differing only in the defects.
    # Without this row, a rule that fires on every file would pass all of the
    # above.
    check("no rule fires anywhere in the intact project",
          not [f for f in data["findings"] if f["path"].startswith("clean/")],
          f"{[(f['rule'], f['path'], f['line']) for f in data['findings'] if f['path'].startswith('clean/')]}")

    # -- exit codes -------------------------------------------------------
    check("findings exit 1", code == 1, f"exit={code}")
    clean_code, clean_data = report(root, "--path", "clean")
    check("a clean selection exits 0 with no findings",
          clean_code == 0 and clean_data is not None and not clean_data["findings"],
          f"exit={clean_code}")
    check("a clean selection still measured something",
          clean_data is not None and clean_data["population"]["scanned"] == 2,
          f"scanned={clean_data['population']['scanned'] if clean_data else None}")
    bad_rule = run(root, "--rule", "M9")
    check("an unknown rule exits 2, distinguishably from a finding",
          bad_rule.returncode == 2, f"exit={bad_rule.returncode}")
    bad_path = run(root, "--path", "no/such/place")
    check("a `--path` that matches nothing exits 2 rather than reporting clean",
          bad_path.returncode == 2, f"exit={bad_path.returncode}")
    outside = subprocess.run([*LAUNCH, str(META_SCRIPT), "check",
                              "--repo", str(pathlib.Path(tmp))],
                             capture_output=True, text=True)
    check("a directory that is not a git checkout exits 2",
          outside.returncode == 2, f"exit={outside.returncode}")

    # -- `--rule` actually restricts --------------------------------------
    only_m3 = report(root, "--rule", "M3")[1]
    check("`--rule` restricts the findings to the rule named",
          only_m3 is not None and {f["rule"] for f in only_m3["findings"]} == {"M3"},
          f"{sorted({f['rule'] for f in only_m3['findings']}) if only_m3 else None}")
    check("`--rule M3` reports the same M3 rows as the full run",
          only_m3 is not None and rows(only_m3, "M3") == rows(data, "M3"))

# --------------------------------------------------------------------------
# the two tables that decide what is a defect
# --------------------------------------------------------------------------
# Both are declarations rather than code, so both are measured by mutating them
# and re-running the rule. `PRODUCERS` being empty is what makes all 66 marker
# pairs orphaned; if the rule ignored it, the emptiness would be decoration.
meta = load_script_module(META_SCRIPT, "metamod")

PAIR = "<!-- aw:meta:demo:start -->\ncontent\n<!-- aw:meta:demo:end -->\n"

out = []
meta.m1_orphan_markers("demo.md", PAIR, out)
check("M1 fires on a pair whose producer is undeclared", len(out) == 1, f"{len(out)}")

shipped_producers = dict(meta.PRODUCERS)
meta.PRODUCERS = {"demo": "some-generator"}
out = []
meta.m1_orphan_markers("demo.md", PAIR, out)
check("M1 stops firing once a producer is declared for that marker",
      out == [], f"{[f.message for f in out]}")
meta.PRODUCERS = shipped_producers
check("the shipped producer table is empty, which is why every marker is orphaned",
      meta.PRODUCERS == {}, f"{meta.PRODUCERS}")

DEAD_LINE = "historical: `aw wi list` was the command\n"
shipped_exempt = dict(meta.DEAD_COMMAND_EXEMPT)

out = []
meta.m2_dead_commands("demo.md", DEAD_LINE, out)
check("M2 fires on an unexempted dead command", len(out) == 1, f"{len(out)}")

meta.DEAD_COMMAND_EXEMPT = {"demo.md": ("historical: `aw wi list` was the command",)}
out = []
meta.m2_dead_commands("demo.md", DEAD_LINE, out)
check("an exemption suppresses exactly the line it quotes", out == [],
      f"{[f.message for f in out]}")

out = []
meta.m2_stale_exemptions({"demo.md"}, {"demo.md": "the sentence was rewritten\n"}, True, out)
check("an exemption that no longer matches is itself a finding",
      len(out) == 1 and "stale exemption" in out[0].message,
      f"{[f.message for f in out]}")

out = []
meta.m2_stale_exemptions(set(), {}, True, out)
check("an exemption naming a file outside the population is a finding",
      len(out) == 1 and "not a tracked META-doc" in out[0].message,
      f"{[f.message for f in out]}")

out = []
meta.m2_stale_exemptions(set(), {}, False, out)
check("under `--path`, an unselected file is not called a stale exemption",
      out == [], f"{[f.message for f in out]}")
meta.DEAD_COMMAND_EXEMPT = shipped_exempt

# --------------------------------------------------------------------------
# one detector, two populations
# --------------------------------------------------------------------------
# `check_plugin.py` runs the same pattern over the skill bodies. Nothing else
# compares the two, so a fix applied to one would silently leave the other
# reading a narrower defect.
PATTERN = re.compile(r"^AW_INVOCATION = re\.compile\((.*)\)$", re.M)
here = PATTERN.search(META_SCRIPT.read_text(encoding="utf-8"))
there = PATTERN.search((PLUGIN_DIR / "verification/check_plugin.py").read_text(encoding="utf-8"))
check("both `AW_INVOCATION` definitions were found", bool(here) and bool(there))
check("`meta.py` and `check_plugin.py` share one `aw <verb>` detector",
      bool(here) and bool(there) and here.group(1) == there.group(1),
      f"meta={here.group(1) if here else None} plugin={there.group(1) if there else None}")

# --------------------------------------------------------------------------
# against the real checkout
# --------------------------------------------------------------------------
# Not a finding count -- that number is meant to fall as the cleanup lands, and
# a gate pinned to it would go red on the fix. What is asserted is that the run
# reached the repository at all, and that the shipped exemption table is still
# describing sentences that exist.
live_code, live = report(REPO)
check("the live run parsed", live is not None, f"exit={live_code}")
if live is not None:
    pop = live["population"]
    check("the live run scanned every tracked META-doc",
          pop["scanned"] == pop["tracked_meta_docs"] and pop["scanned"] > 100,
          f"{pop}")
    check("the live run found the project population",
          pop["project_readmes"] > 20, f"project_readmes={pop['project_readmes']}")
    stale = [f for f in live["findings"]
             if "stale exemption" in f["message"] or "not a tracked META-doc" in f["message"]]
    check("every shipped exemption still quotes a line that exists",
          not stale, f"{[(f['path'], f['message']) for f in stale]}")

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)

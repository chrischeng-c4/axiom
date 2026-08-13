#!/usr/bin/env python3
"""Differential: the crate's own rules vs the port, over every live change body.

`check_change_schema.py` replays the 18 cases the crate wrote for itself. Those
are the cases whose answers were known in advance, which is exactly the input
distribution a port is least likely to fail on. This runs the two readings
against the bodies humans actually wrote -- hundreds of them, in every shape
from clean GHAN to legacy six-section to prose -- and demands the error lists
match verbatim, in order.

What that buys is breadth, not depth, and the run says which it got. Validation
short-circuits at the structural tier, so a body missing an H2 is refused before
any section rule runs; live, that is most of them. The reach assertion prints
the split rather than letting a five-figure error count imply the per-section
rules were compared where they fire.

The oracle is not my reading of `ghan.rs`. It is `ghan.rs`, compiled and
executed:

  1. the rule half of the file is extracted mechanically (drop `use super::Issue`
     and `validate_ghan_body`, the only two items that reach into the crate),
  2. a `main` that reads bodies and prints errors is appended,
  3. `rustc` builds it, and both sides run over the same bodies.

Re-derived from source on every run, so the harness cannot go stale the way a
transcribed copy would. The extractor asserts what it removed and what survived;
an extraction that silently produced an empty file would otherwise agree with
everything.

**No tracker writes.** This matters enough to be a design constraint rather than
a side effect: `aw wi validate`, the obvious oracle, is not read-only -- its
failure path calls `backend.update()` with `validation_errors`, so pointing it
at hundreds of work items is a write sweep wearing a measurement's clothes.
Reading bodies through `gh issue list` is a plain GET, and the run is bracketed
by an `updatedAt` census so read-only is measured rather than asserted.

Because the harness calls `validate_ghan_sections` directly, both sides compute
the *same* function and the differential carries no excluded error class at all.
`validate_ghan_body`'s extra `looks_too_large_for_atomic_wi` rule is outside the
comparison on both sides, not tolerated on one.

Usage:  measure_change_agreement.py [--limit N] [--repo owner/name]
"""
import argparse
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import GHAN_RS, TRACKER_REPO, load_change_module  # noqa: E402

parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
parser.add_argument("--limit", type=int, default=1000)
parser.add_argument("--repo", default=TRACKER_REPO)
parser.add_argument("--label", default="type:change")
args = parser.parse_args()

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


# --------------------------------------------------------------------------
# 1. Extract the crate's rules into a standalone program
# --------------------------------------------------------------------------

SRC = GHAN_RS.read_text(encoding="utf-8")


def drop_item(src, signature):
    """Remove one `fn` and the doc comment attached to it, by brace matching."""
    start = src.index(signature)
    # Walk back over the doc comment / attributes glued to this item.
    head = src[:start].rstrip("\n").splitlines()
    while head and (head[-1].lstrip().startswith("///") or head[-1].lstrip().startswith("#[")):
        head.pop()
    depth, end = 0, None
    for i in range(start, len(src)):
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    if end is None:
        raise SystemExit(f"error: no closing brace for {signature!r}")
    return "\n".join(head) + "\n" + src[end:].lstrip("\n")


rules = SRC.replace("use super::Issue;\n", "")
rules = drop_item(rules, "pub fn validate_ghan_body(")

# The *item* must be gone, not every mention of it: `validate_ghan_sections`'s
# own doc comment names `validate_ghan_body` as the caller it was split from,
# and a doc comment cannot fail to compile.
check("the extractor removed exactly the two crate-coupled items",
      "super::Issue" not in rules and "fn validate_ghan_body(" not in rules
      and "planner::looks_too_large" not in rules and len(rules) < len(SRC),
      f"{len(SRC)}B -> {len(rules)}B")
check("the extractor kept the rule surface",
      all(name in rules for name in
          ("fn validate_ghan_sections", "fn validate_goal", "fn validate_how",
           "fn validate_acceptance", "fn validate_never", "fn body_shape")),
      "all six entry points present")

HARNESS_MAIN = r'''
fn main() {
    let dir = std::env::args().nth(1).expect("usage: harness <dir>");
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut out = String::new();
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let body = std::fs::read_to_string(&path).expect("read body");
        let shape = match body_shape(&body) {
            WiBodyShape::Unstructured => "unstructured",
            WiBodyShape::Legacy => "legacy",
            WiBodyShape::Ghan => "ghan",
            WiBodyShape::Mixed => "mixed",
        };
        let errors = validate_ghan_sections(&body);
        out.push_str(&format!("#{}\t{}\t{}\n", name, shape, errors.len()));
        for e in errors {
            out.push_str(&format!("E{}\n", e.replace('\n', " ")));
        }
    }
    print!("{}", out);
}
'''

if fails:
    print(f"\n=> RED ({len(fails)} failure(s))")
    sys.exit(1)

work = pathlib.Path(tempfile.mkdtemp(prefix="ghan-diff-"))
try:
    source = work / "harness.rs"
    source.write_text("#![allow(dead_code)]\n" + rules + HARNESS_MAIN, encoding="utf-8")
    binary = work / "harness"
    build = subprocess.run(
        ["rustc", "--edition", "2021", "-O", "-o", str(binary), str(source)],
        capture_output=True, text=True,
    )
    check("the crate's rules compile standalone", build.returncode == 0,
          build.stderr.strip().splitlines()[-1] if build.returncode else "")
    if build.returncode != 0:
        print(build.stderr)
        print(f"\n=> RED ({len(fails)} failure(s))")
        sys.exit(1)

    # ----------------------------------------------------------------------
    # 2. Fetch the population -- read-only, and bracketed to prove it
    # ----------------------------------------------------------------------

    def gh_json(*argv):
        r = subprocess.run(["gh", *argv], capture_output=True, text=True)
        if r.returncode != 0:
            raise SystemExit(f"error: gh {' '.join(argv)} failed:\n{r.stderr.strip()}")
        return json.loads(r.stdout)

    listing = ["issue", "list", "--repo", args.repo, "--label", args.label,
               "--state", "all", "--limit", str(args.limit)]
    issues = gh_json(*listing, "--json", "number,body,updatedAt")
    check("the population is non-empty", bool(issues), f"{len(issues)} `{args.label}` work item(s)")
    if not issues:
        print(f"\n=> RED ({len(fails)} failure(s))")
        sys.exit(1)

    before = {str(i["number"]): i["updatedAt"] for i in issues}

    bodies = work / "bodies"
    bodies.mkdir()
    for issue in issues:
        (bodies / f"{issue['number']}.md").write_text(issue.get("body") or "", encoding="utf-8")

    # ----------------------------------------------------------------------
    # 3. Run both readings over the same bodies
    # ----------------------------------------------------------------------

    run = subprocess.run([str(binary), str(bodies)], capture_output=True, text=True)
    check("the harness ran over every body", run.returncode == 0, run.stderr.strip()[:200])
    if run.returncode != 0:
        print(f"\n=> RED ({len(fails)} failure(s))")
        sys.exit(1)

    crate = {}
    current = None
    for line in run.stdout.splitlines():
        if line.startswith("#"):
            number, shape, _count = line[1:].split("\t")
            current = {"shape": shape, "errors": []}
            crate[number] = current
        elif line.startswith("E") and current is not None:
            current["errors"].append(line[1:])

    check("the harness reported one verdict per body", len(crate) == len(issues),
          f"harness={len(crate)} bodies={len(issues)}")

    mod = load_change_module()
    shape_mismatch, error_mismatch = [], []
    for issue in issues:
        number = str(issue["number"])
        body = issue.get("body") or ""
        theirs = crate.get(number, {"shape": "<missing>", "errors": []})
        mine_shape = mod.body_shape(body)
        mine_errors = [e.replace("\n", " ") for e in mod.validate_body(body)]
        if mine_shape != theirs["shape"]:
            shape_mismatch.append((number, theirs["shape"], mine_shape))
        if mine_errors != theirs["errors"]:
            error_mismatch.append((number, theirs["errors"], mine_errors))

    shapes = {}
    for entry in crate.values():
        shapes[entry["shape"]] = shapes.get(entry["shape"], 0) + 1
    refused = sum(1 for e in crate.values() if e["errors"])
    total_errors = sum(len(e["errors"]) for e in crate.values())

    print()
    check("every body classifies to the same shape", not shape_mismatch,
          f"{len(issues)} bodies, shapes={shapes}, mismatches={shape_mismatch[:5]}")
    check("every body produces the identical error list", not error_mismatch,
          f"{refused}/{len(issues)} refused, {total_errors} error string(s) compared")
    for number, theirs, mine in error_mismatch[:5]:
        print(f"     #{number}\n       crate: {theirs}\n       port:  {mine}")

    # The differential is only as strong as the disagreement it could have
    # seen. A population that produced no errors at all would compare two empty
    # lists several hundred times and report perfect agreement.
    check("the comparison was non-vacuous: the crate refused bodies here",
          refused > 0 and total_errors >= len(crate),
          f"{refused} refused body(ies), {total_errors} error strings")

    # ...and only as strong as the tier it reached. `validate_ghan_sections`
    # short-circuits: a body missing an H2 or carrying an unexpected one is
    # refused structurally and never reaches `validate_goal` and its three
    # siblings. A large error count is therefore not evidence of depth -- it is
    # mostly evidence that legacy bodies are legacy. Report the split so the
    # number cannot be read as more than it is, and refuse a run that never got
    # past the structural tier at all, which would leave the per-section rules
    # uncompared while still printing several thousand agreeing strings.
    STRUCTURAL = ("ghan: missing required", "ghan: unexpected H2")
    deep = {n: e for n, e in crate.items()
            if not any(s.startswith(p) for s in e["errors"] for p in STRUCTURAL)}
    deep_errors = sum(len(e["errors"]) for e in deep.values())
    check("the differential reached the per-section rules", bool(deep),
          f"{len(deep)}/{len(crate)} bodies past the short-circuit, "
          f"{deep_errors} per-section error string(s)")
    if deep and deep_errors == 0:
        print("     note: every body that reached the per-section rules passed them, so")
        print("           those rules are compared here on their non-firing path only.")
        print("           Their firing path is covered by check_change_schema.py's replay")
        print("           of the crate's own tests.")

    # ----------------------------------------------------------------------
    # 4. Prove the run was read-only
    # ----------------------------------------------------------------------

    after_issues = gh_json(*listing, "--json", "number,updatedAt")
    after = {str(i["number"]): i["updatedAt"] for i in after_issues}
    touched = sorted(n for n, stamp in before.items() if after.get(n) != stamp)
    check("no work item was modified by this measurement", not touched,
          f"{len(before)} updatedAt stamps unchanged" if not touched else f"touched={touched}")

finally:
    shutil.rmtree(work, ignore_errors=True)

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)

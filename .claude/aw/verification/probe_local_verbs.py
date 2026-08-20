#!/usr/bin/env python3
"""Local-only probes for epic.py: adopt guards, idempotency, id parsing.

None of these reach the tracker. `adopt` is the verb worth guarding hardest:
it renames a file, so its refusals -- a path outside the staging tree, and a
target that already exists with different content -- are the difference between
a rename and a silent overwrite of somebody else's staged body.
"""
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SCRIPT, load_epic_module  # noqa: E402

ok = []


def run(*a):
    r = subprocess.run([sys.executable, str(SCRIPT), *a], capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip()


def check(cond, msg):
    print(("PASS " if cond else "FAIL ") + msg)
    ok.append(bool(cond))


stage = pathlib.Path(run("bodydir")[1])

# --- adopt refuses paths outside the staging tree (negative control) -------
with tempfile.TemporaryDirectory() as tmp:
    outside = pathlib.Path(tmp) / "epic-adopt-outside.md"
    outside.write_text("body\n")
    code, out = run("adopt", str(outside), "9999")
    check(code != 0, f"adopt exits non-zero for a path outside the tree (exit={code})")
    check(outside.is_file() and not (pathlib.Path(tmp) / "9999.md").exists(),
          "the outside file was not renamed")
    print("   ->", out.splitlines()[-1] if out else "")

# --- adopt happy path + idempotency ---------------------------------------
slug = stage / "adopt-probe.md"
slug.write_text("body\n")
code1, _ = run("adopt", str(slug), "9998")
target = stage / "9998.md"
check(code1 == 0 and target.is_file() and not slug.exists(),
      f"adopt renamed the staged body (exit={code1})")
code2, out2 = run("adopt", str(target), "9998")
check(code2 == 0, f"adopt is idempotent: second run is a no-op (exit={code2})")
print("   ->", out2)

# --- adopt refuses to clobber a different file ----------------------------
other = stage / "collide-probe.md"
other.write_text("different\n")
code3, _ = run("adopt", str(other), "9998")
check(code3 != 0 and other.is_file() and target.read_text() == "body\n",
      f"adopt refuses to overwrite an existing different file (exit={code3})")
other.unlink()
target.unlink()

# --- id parsing ------------------------------------------------------------
f = load_epic_module().issue_number_from_create_output
check(f("https://github.com/chrischeng-c4/axiom/issues/3601\n") == "3601", "URL -> id")
check(f("Creating issue in chrischeng-c4/axiom\n\n"
        "https://github.com/chrischeng-c4/axiom/issues/42") == "42",
      "multi-line gh output still yields the id")
check(f("something went sideways") is None, "input with no number returns None")

print("\n=> " + ("GREEN" if all(ok) else f"RED ({ok.count(False)} failure(s))"))
sys.exit(0 if all(ok) else 1)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "double_star_non_recursive_stays_one_level"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: with recursive=False, '**/*.txt' behaves like a single '*' segment and does NOT reach the deep 'sub/deep/c.txt' entry"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for rel in ("a.txt", os.path.join("sub", "b.txt"), os.path.join("sub", "deep", "c.txt")):
        full = os.path.join(d, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as fh:
            fh.write("")
    rec = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=True))
    norec = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=False))
    assert len(rec) == 3, f"recursive count = {len(rec)!r}"
    deep = [p for p in norec if "deep" in p]
    assert deep == [], f"non-recursive no deep = {deep!r}"

print("double_star_non_recursive_stays_one_level OK")

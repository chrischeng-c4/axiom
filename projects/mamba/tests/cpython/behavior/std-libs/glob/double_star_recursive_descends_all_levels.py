# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "double_star_recursive_descends_all_levels"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: with recursive=True, '**/*.txt' descends every level: a tree {a.txt, sub/b.txt, sub/deep/c.txt} yields all three relative paths"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for rel in ("a.txt", os.path.join("sub", "b.txt"), os.path.join("sub", "deep", "c.txt")):
        full = os.path.join(d, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=True))
    rels = sorted(os.path.relpath(p, d) for p in results)
    assert rels == sorted(["a.txt", os.path.join("sub", "b.txt"),
                           os.path.join("sub", "deep", "c.txt")]), f"recursive = {rels!r}"

print("double_star_recursive_descends_all_levels OK")

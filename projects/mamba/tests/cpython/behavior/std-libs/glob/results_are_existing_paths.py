# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "results_are_existing_paths"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: every path glob returns actually exists on disk: os.path.exists is True for each result of glob('*.txt') in a populated temp dir"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.txt"))
    assert len(results) == 2, f"count = {len(results)!r}"
    assert all(os.path.exists(p) for p in results), f"all exist = {results!r}"

print("results_are_existing_paths OK")

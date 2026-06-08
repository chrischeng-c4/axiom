# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob_returns_str_paths"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: a str pattern yields str results: every element of glob('*') in a temp dir is a str"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*"))
    assert len(results) == 2, f"count = {len(results)!r}"
    assert {type(p).__name__ for p in results} == {"str"}, f"types = {results!r}"

print("glob_returns_str_paths OK")

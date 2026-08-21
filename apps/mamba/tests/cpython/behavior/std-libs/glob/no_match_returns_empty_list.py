# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "no_match_returns_empty_list"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: a pattern with no matching entry returns an empty list (not None, not a raise): glob('*.xyz') in a dir with no .xyz files == []"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.xyz"))
    assert results == [], f"no match = {results!r}"

print("no_match_returns_empty_list OK")

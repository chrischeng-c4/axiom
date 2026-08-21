# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "qmark_matches_exactly_one_char"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: ? matches exactly one character, not zero or many: '?.txt' over {a.txt,ab.txt,abc.txt} matches only 'a.txt'"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "ab.txt", "abc.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "?.txt")))
    bases = [os.path.basename(p) for p in results]
    assert bases == ["a.txt"], f"?.txt = {bases!r}"

print("qmark_matches_exactly_one_char OK")

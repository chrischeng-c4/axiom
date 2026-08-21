# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "char_class_matches_set"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: the character class [abc] matches any one of the listed chars: '[abc].txt' over {a,b,c,d}.txt yields a.txt,b.txt,c.txt and not d.txt"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt", "d.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "[abc].txt")))
    bases = [os.path.basename(p) for p in results]
    assert bases == ["a.txt", "b.txt", "c.txt"], f"[abc].txt = {bases!r}"

print("char_class_matches_set OK")

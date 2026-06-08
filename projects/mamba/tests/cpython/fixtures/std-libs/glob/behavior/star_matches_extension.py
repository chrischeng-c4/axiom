# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "star_matches_extension"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: glob('*.txt') returns a list of the matching paths; in a temp dir of {a.txt,b.txt,c.py} the *.txt pattern yields exactly the two .txt files (sorted basenames ['a.txt','b.txt'])"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.txt"))
    assert isinstance(results, list), f"glob type = {type(results)!r}"
    bases = sorted(os.path.basename(p) for p in results)
    assert bases == ["a.txt", "b.txt"], f"*.txt = {bases!r}"

print("star_matches_extension OK")

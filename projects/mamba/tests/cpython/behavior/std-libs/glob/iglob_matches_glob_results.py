# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "iglob_matches_glob_results"
# subject = "glob.iglob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.iglob: iglob and glob produce the same set of paths for one pattern: sorted(glob('*.py')) == sorted(iglob('*.py')) over {x,y,z}.py"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("x.py", "y.py", "z.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    pattern = os.path.join(d, "*.py")
    g = sorted(glob.glob(pattern))
    ig = sorted(glob.iglob(pattern))
    assert g == ig, f"glob {g} vs iglob {ig}"
    assert len(g) == 3, f"count = {len(g)!r}"

print("iglob_matches_glob_results OK")

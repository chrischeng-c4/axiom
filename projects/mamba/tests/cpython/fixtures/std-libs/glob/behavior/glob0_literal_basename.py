# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob0_literal_basename"
# subject = "glob.glob0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob0: glob0(dirname, literal) returns [literal] when the named entry exists in dirname and [] when it does not (literal-basename helper)"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with open(os.path.join(d, "alpha.txt"), "w") as fh:
        fh.write("")
    assert glob.glob0(d, "alpha.txt") == ["alpha.txt"], "literal hit"
    assert glob.glob0(d, "missing.zzz") == [], "literal miss"

print("glob0_literal_basename OK")

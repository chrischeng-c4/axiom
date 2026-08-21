# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "match_namedtuple_exists"
# subject = "difflib.Match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.Match: match_namedtuple_exists (surface)."""
import difflib

assert hasattr(difflib.Match, "_fields")
print("match_namedtuple_exists OK")

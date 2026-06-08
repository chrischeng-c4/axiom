# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_sequence_matcher_is_present"
# subject = "difflib.SequenceMatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.SequenceMatcher: api_sequence_matcher_is_present (surface)."""
import difflib

assert hasattr(difflib, "SequenceMatcher")
print("api_sequence_matcher_is_present OK")

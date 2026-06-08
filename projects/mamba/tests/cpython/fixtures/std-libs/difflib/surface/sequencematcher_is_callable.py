# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "sequencematcher_is_callable"
# subject = "difflib.SequenceMatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: sequencematcher_is_callable (surface)."""
import difflib

assert callable(difflib.SequenceMatcher)
print("sequencematcher_is_callable OK")

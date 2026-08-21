# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "accumulate_is_callable"
# subject = "itertools.accumulate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.accumulate: accumulate_is_callable (surface)."""
import itertools

assert callable(itertools.accumulate)
print("accumulate_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "cycle_is_callable"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.cycle: cycle_is_callable (surface)."""
import itertools

assert callable(itertools.cycle)
print("cycle_is_callable OK")

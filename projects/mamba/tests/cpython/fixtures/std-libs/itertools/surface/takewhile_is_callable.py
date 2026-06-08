# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "takewhile_is_callable"
# subject = "itertools.takewhile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.takewhile: takewhile_is_callable (surface)."""
import itertools

assert callable(itertools.takewhile)
print("takewhile_is_callable OK")

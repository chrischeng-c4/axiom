# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "tee_is_callable"
# subject = "itertools.tee"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.tee: tee_is_callable (surface)."""
import itertools

assert callable(itertools.tee)
print("tee_is_callable OK")

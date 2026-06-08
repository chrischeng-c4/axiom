# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "repeat_is_callable"
# subject = "itertools.repeat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.repeat: repeat_is_callable (surface)."""
import itertools

assert callable(itertools.repeat)
print("repeat_is_callable OK")

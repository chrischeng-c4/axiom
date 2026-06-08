# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "dropwhile_is_callable"
# subject = "itertools.dropwhile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.dropwhile: dropwhile_is_callable (surface)."""
import itertools

assert callable(itertools.dropwhile)
print("dropwhile_is_callable OK")

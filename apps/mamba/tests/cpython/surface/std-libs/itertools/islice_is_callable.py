# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "islice_is_callable"
# subject = "itertools.islice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.islice: islice_is_callable (surface)."""
import itertools

assert callable(itertools.islice)
print("islice_is_callable OK")

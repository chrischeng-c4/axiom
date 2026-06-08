# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "zip_longest_is_callable"
# subject = "itertools.zip_longest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest_is_callable (surface)."""
import itertools

assert callable(itertools.zip_longest)
print("zip_longest_is_callable OK")

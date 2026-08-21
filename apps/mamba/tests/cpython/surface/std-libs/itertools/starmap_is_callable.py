# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "starmap_is_callable"
# subject = "itertools.starmap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.starmap: starmap_is_callable (surface)."""
import itertools

assert callable(itertools.starmap)
print("starmap_is_callable OK")

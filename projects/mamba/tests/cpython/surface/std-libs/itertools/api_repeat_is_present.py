# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_repeat_is_present"
# subject = "itertools.repeat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.repeat: api_repeat_is_present (surface)."""
import itertools

assert hasattr(itertools, "repeat")
print("api_repeat_is_present OK")

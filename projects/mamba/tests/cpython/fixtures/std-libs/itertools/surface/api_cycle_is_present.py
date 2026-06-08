# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_cycle_is_present"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.cycle: api_cycle_is_present (surface)."""
import itertools

assert hasattr(itertools, "cycle")
print("api_cycle_is_present OK")

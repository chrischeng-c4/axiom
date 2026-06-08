# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "api_deepcopy_is_present"
# subject = "copy.deepcopy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copy.deepcopy: api_deepcopy_is_present (surface)."""
import copy

assert hasattr(copy, "deepcopy")
print("api_deepcopy_is_present OK")

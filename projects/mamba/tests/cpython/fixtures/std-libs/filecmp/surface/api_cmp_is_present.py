# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "surface"
# case = "api_cmp_is_present"
# subject = "filecmp.cmp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""filecmp.cmp: api_cmp_is_present (surface)."""
import filecmp

assert hasattr(filecmp, "cmp")
print("api_cmp_is_present OK")

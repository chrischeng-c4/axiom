# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "surface"
# case = "api_dircmp_is_present"
# subject = "filecmp.dircmp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""filecmp.dircmp: api_dircmp_is_present (surface)."""
import filecmp

assert hasattr(filecmp, "dircmp")
print("api_dircmp_is_present OK")

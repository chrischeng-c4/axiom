# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "surface"
# case = "api_cmpfiles_is_present"
# subject = "filecmp.cmpfiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""filecmp.cmpfiles: api_cmpfiles_is_present (surface)."""
import filecmp

assert hasattr(filecmp, "cmpfiles")
print("api_cmpfiles_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "surface"
# case = "api_default_ignores_is_present"
# subject = "filecmp.DEFAULT_IGNORES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""filecmp.DEFAULT_IGNORES: api_default_ignores_is_present (surface)."""
import filecmp

assert hasattr(filecmp, "DEFAULT_IGNORES")
print("api_default_ignores_is_present OK")

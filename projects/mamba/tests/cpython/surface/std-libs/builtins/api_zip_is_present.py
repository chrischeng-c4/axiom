# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_zip_is_present"
# subject = "builtins.zip"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.zip: api_zip_is_present (surface)."""
import builtins

assert hasattr(builtins, "zip")
print("api_zip_is_present OK")

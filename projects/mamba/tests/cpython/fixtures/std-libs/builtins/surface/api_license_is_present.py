# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_license_is_present"
# subject = "builtins.license"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.license: api_license_is_present (surface)."""
import builtins

assert hasattr(builtins, "license")
print("api_license_is_present OK")

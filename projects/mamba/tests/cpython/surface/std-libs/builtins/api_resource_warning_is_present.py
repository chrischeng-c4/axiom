# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_resource_warning_is_present"
# subject = "builtins.ResourceWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ResourceWarning: api_resource_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "ResourceWarning")
print("api_resource_warning_is_present OK")

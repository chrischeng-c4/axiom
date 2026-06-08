# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_map_is_present"
# subject = "builtins.map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.map: api_map_is_present (surface)."""
import builtins

assert hasattr(builtins, "map")
print("api_map_is_present OK")

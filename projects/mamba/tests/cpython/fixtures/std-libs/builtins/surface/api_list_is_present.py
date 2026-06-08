# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_list_is_present"
# subject = "builtins.list"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.list: api_list_is_present (surface)."""
import builtins

assert hasattr(builtins, "list")
print("api_list_is_present OK")

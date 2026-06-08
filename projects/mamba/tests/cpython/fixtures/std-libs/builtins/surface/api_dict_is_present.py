# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_dict_is_present"
# subject = "builtins.dict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.dict: api_dict_is_present (surface)."""
import builtins

assert hasattr(builtins, "dict")
print("api_dict_is_present OK")

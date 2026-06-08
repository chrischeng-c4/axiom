# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_len_is_present"
# subject = "builtins.len"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.len: api_len_is_present (surface)."""
import builtins

assert hasattr(builtins, "len")
print("api_len_is_present OK")

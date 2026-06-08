# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_dir_is_present"
# subject = "builtins.dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.dir: api_dir_is_present (surface)."""
import builtins

assert hasattr(builtins, "dir")
print("api_dir_is_present OK")

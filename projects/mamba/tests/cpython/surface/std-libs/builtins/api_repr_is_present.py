# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_repr_is_present"
# subject = "builtins.repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.repr: api_repr_is_present (surface)."""
import builtins

assert hasattr(builtins, "repr")
print("api_repr_is_present OK")

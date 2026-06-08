# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_frozenset_is_present"
# subject = "builtins.frozenset"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.frozenset: api_frozenset_is_present (surface)."""
import builtins

assert hasattr(builtins, "frozenset")
print("api_frozenset_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_id_is_present"
# subject = "builtins.id"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.id: api_id_is_present (surface)."""
import builtins

assert hasattr(builtins, "id")
print("api_id_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_next_is_present"
# subject = "builtins.next"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.next: api_next_is_present (surface)."""
import builtins

assert hasattr(builtins, "next")
print("api_next_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_round_is_present"
# subject = "builtins.round"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.round: api_round_is_present (surface)."""
import builtins

assert hasattr(builtins, "round")
print("api_round_is_present OK")

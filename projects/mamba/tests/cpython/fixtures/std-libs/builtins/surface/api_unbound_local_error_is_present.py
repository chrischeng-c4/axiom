# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_unbound_local_error_is_present"
# subject = "builtins.UnboundLocalError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UnboundLocalError: api_unbound_local_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "UnboundLocalError")
print("api_unbound_local_error_is_present OK")

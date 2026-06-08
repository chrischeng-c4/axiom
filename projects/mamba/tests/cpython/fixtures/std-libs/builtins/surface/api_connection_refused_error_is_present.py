# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_connection_refused_error_is_present"
# subject = "builtins.ConnectionRefusedError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ConnectionRefusedError: api_connection_refused_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ConnectionRefusedError")
print("api_connection_refused_error_is_present OK")

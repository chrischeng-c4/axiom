# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_connection_aborted_error_is_present"
# subject = "builtins.ConnectionAbortedError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ConnectionAbortedError: api_connection_aborted_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ConnectionAbortedError")
print("api_connection_aborted_error_is_present OK")

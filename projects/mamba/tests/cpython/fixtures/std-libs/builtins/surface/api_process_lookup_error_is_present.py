# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_process_lookup_error_is_present"
# subject = "builtins.ProcessLookupError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ProcessLookupError: api_process_lookup_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ProcessLookupError")
print("api_process_lookup_error_is_present OK")

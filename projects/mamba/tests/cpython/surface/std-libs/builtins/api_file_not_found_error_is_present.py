# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_file_not_found_error_is_present"
# subject = "builtins.FileNotFoundError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.FileNotFoundError: api_file_not_found_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "FileNotFoundError")
print("api_file_not_found_error_is_present OK")

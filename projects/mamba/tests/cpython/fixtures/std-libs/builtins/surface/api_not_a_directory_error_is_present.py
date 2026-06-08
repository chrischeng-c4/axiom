# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_not_a_directory_error_is_present"
# subject = "builtins.NotADirectoryError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.NotADirectoryError: api_not_a_directory_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "NotADirectoryError")
print("api_not_a_directory_error_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_file_exists_error_is_present"
# subject = "builtins.FileExistsError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.FileExistsError: api_file_exists_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "FileExistsError")
print("api_file_exists_error_is_present OK")

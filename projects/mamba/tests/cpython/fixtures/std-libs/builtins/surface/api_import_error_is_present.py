# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_import_error_is_present"
# subject = "builtins.ImportError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ImportError: api_import_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ImportError")
print("api_import_error_is_present OK")

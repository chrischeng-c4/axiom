# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_import_warning_is_present"
# subject = "builtins.ImportWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ImportWarning: api_import_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "ImportWarning")
print("api_import_warning_is_present OK")

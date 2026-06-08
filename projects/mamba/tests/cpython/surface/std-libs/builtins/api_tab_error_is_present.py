# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_tab_error_is_present"
# subject = "builtins.TabError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.TabError: api_tab_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "TabError")
print("api_tab_error_is_present OK")

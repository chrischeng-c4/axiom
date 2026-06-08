# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_user_warning_is_present"
# subject = "builtins.UserWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UserWarning: api_user_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "UserWarning")
print("api_user_warning_is_present OK")

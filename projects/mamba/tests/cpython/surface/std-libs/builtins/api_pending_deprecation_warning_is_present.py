# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_pending_deprecation_warning_is_present"
# subject = "builtins.PendingDeprecationWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.PendingDeprecationWarning: api_pending_deprecation_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "PendingDeprecationWarning")
print("api_pending_deprecation_warning_is_present OK")

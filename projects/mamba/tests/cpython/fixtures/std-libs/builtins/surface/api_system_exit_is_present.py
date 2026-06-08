# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_system_exit_is_present"
# subject = "builtins.SystemExit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.SystemExit: api_system_exit_is_present (surface)."""
import builtins

assert hasattr(builtins, "SystemExit")
print("api_system_exit_is_present OK")

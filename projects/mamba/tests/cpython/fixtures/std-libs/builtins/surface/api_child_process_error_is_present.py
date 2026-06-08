# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_child_process_error_is_present"
# subject = "builtins.ChildProcessError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ChildProcessError: api_child_process_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ChildProcessError")
print("api_child_process_error_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_quit_is_present"
# subject = "builtins.quit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.quit: api_quit_is_present (surface)."""
import builtins

assert hasattr(builtins, "quit")
print("api_quit_is_present OK")

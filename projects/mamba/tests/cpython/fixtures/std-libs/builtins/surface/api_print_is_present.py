# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_print_is_present"
# subject = "builtins.print"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.print: api_print_is_present (surface)."""
import builtins

assert hasattr(builtins, "print")
print("api_print_is_present OK")

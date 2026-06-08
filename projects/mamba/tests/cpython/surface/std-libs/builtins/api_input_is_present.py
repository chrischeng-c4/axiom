# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_input_is_present"
# subject = "builtins.input"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.input: api_input_is_present (surface)."""
import builtins

assert hasattr(builtins, "input")
print("api_input_is_present OK")

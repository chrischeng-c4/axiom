# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_generator_exit_is_present"
# subject = "builtins.GeneratorExit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.GeneratorExit: api_generator_exit_is_present (surface)."""
import builtins

assert hasattr(builtins, "GeneratorExit")
print("api_generator_exit_is_present OK")

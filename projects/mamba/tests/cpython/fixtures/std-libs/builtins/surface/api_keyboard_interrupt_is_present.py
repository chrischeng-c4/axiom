# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_keyboard_interrupt_is_present"
# subject = "builtins.KeyboardInterrupt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.KeyboardInterrupt: api_keyboard_interrupt_is_present (surface)."""
import builtins

assert hasattr(builtins, "KeyboardInterrupt")
print("api_keyboard_interrupt_is_present OK")

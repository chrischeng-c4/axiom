# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_activate_stack_trampoline_is_present"
# subject = "sys.activate_stack_trampoline"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.activate_stack_trampoline: api_activate_stack_trampoline_is_present (surface)."""
import sys

assert hasattr(sys, "activate_stack_trampoline")
print("api_activate_stack_trampoline_is_present OK")

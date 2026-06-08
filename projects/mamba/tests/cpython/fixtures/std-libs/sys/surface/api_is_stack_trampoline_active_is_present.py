# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_is_stack_trampoline_active_is_present"
# subject = "sys.is_stack_trampoline_active"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.is_stack_trampoline_active: api_is_stack_trampoline_active_is_present (surface)."""
import sys

assert hasattr(sys, "is_stack_trampoline_active")
print("api_is_stack_trampoline_active_is_present OK")

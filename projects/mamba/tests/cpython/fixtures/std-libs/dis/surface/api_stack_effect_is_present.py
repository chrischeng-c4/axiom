# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_stack_effect_is_present"
# subject = "dis.stack_effect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.stack_effect: api_stack_effect_is_present (surface)."""
import dis

assert hasattr(dis, "stack_effect")
print("api_stack_effect_is_present OK")

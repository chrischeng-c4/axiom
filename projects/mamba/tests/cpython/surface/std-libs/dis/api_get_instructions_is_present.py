# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_get_instructions_is_present"
# subject = "dis.get_instructions"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.get_instructions: api_get_instructions_is_present (surface)."""
import dis

assert hasattr(dis, "get_instructions")
print("api_get_instructions_is_present OK")

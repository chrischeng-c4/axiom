# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_instruction_is_present"
# subject = "dis.Instruction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.Instruction: api_instruction_is_present (surface)."""
import dis

assert hasattr(dis, "Instruction")
print("api_instruction_is_present OK")

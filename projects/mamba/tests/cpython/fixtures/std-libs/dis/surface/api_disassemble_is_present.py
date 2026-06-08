# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_disassemble_is_present"
# subject = "dis.disassemble"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.disassemble: api_disassemble_is_present (surface)."""
import dis

assert hasattr(dis, "disassemble")
print("api_disassemble_is_present OK")

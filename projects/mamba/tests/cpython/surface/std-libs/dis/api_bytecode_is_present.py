# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_bytecode_is_present"
# subject = "dis.Bytecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.Bytecode: api_bytecode_is_present (surface)."""
import dis

assert hasattr(dis, "Bytecode")
print("api_bytecode_is_present OK")

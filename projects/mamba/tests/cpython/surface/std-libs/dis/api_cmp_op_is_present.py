# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_cmp_op_is_present"
# subject = "dis.cmp_op"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.cmp_op: api_cmp_op_is_present (surface)."""
import dis

assert hasattr(dis, "cmp_op")
print("api_cmp_op_is_present OK")

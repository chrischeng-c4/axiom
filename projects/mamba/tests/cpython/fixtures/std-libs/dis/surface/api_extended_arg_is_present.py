# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_extended_arg_is_present"
# subject = "dis.EXTENDED_ARG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.EXTENDED_ARG: api_extended_arg_is_present (surface)."""
import dis

assert hasattr(dis, "EXTENDED_ARG")
print("api_extended_arg_is_present OK")

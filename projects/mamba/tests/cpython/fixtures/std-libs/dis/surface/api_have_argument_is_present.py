# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_have_argument_is_present"
# subject = "dis.HAVE_ARGUMENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.HAVE_ARGUMENT: api_have_argument_is_present (surface)."""
import dis

assert hasattr(dis, "HAVE_ARGUMENT")
print("api_have_argument_is_present OK")

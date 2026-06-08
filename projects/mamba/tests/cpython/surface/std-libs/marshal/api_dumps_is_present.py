# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "surface"
# case = "api_dumps_is_present"
# subject = "marshal.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""marshal.dumps: api_dumps_is_present (surface)."""
import marshal

assert hasattr(marshal, "dumps")
print("api_dumps_is_present OK")

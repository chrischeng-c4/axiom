# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "surface"
# case = "api_dump_is_present"
# subject = "marshal.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""marshal.dump: api_dump_is_present (surface)."""
import marshal

assert hasattr(marshal, "dump")
print("api_dump_is_present OK")

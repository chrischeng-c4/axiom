# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_seek_set_is_present"
# subject = "io.SEEK_SET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.SEEK_SET: api_seek_set_is_present (surface)."""
import io

assert hasattr(io, "SEEK_SET")
print("api_seek_set_is_present OK")

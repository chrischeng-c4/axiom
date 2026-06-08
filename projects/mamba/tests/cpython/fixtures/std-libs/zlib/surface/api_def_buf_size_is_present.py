# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_def_buf_size_is_present"
# subject = "zlib.DEF_BUF_SIZE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.DEF_BUF_SIZE: api_def_buf_size_is_present (surface)."""
import zlib

assert hasattr(zlib, "DEF_BUF_SIZE")
print("api_def_buf_size_is_present OK")

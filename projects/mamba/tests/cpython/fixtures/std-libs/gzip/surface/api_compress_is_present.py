# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "api_compress_is_present"
# subject = "gzip.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gzip.compress: api_compress_is_present (surface)."""
import gzip

assert hasattr(gzip, "compress")
print("api_compress_is_present OK")

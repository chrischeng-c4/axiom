# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "api_decompress_is_present"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gzip.decompress: api_decompress_is_present (surface)."""
import gzip

assert hasattr(gzip, "decompress")
print("api_decompress_is_present OK")

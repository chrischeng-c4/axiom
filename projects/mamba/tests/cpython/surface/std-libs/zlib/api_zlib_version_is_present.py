# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_zlib_version_is_present"
# subject = "zlib.ZLIB_VERSION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.ZLIB_VERSION: api_zlib_version_is_present (surface)."""
import zlib

assert hasattr(zlib, "ZLIB_VERSION")
print("api_zlib_version_is_present OK")

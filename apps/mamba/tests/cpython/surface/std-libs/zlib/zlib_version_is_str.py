# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "zlib_version_is_str"
# subject = "zlib.ZLIB_VERSION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.ZLIB_VERSION: zlib_version_is_str (surface)."""
import zlib

assert type(zlib.ZLIB_VERSION).__name__ == "str"
print("zlib_version_is_str OK")

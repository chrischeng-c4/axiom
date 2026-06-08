# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "version_constants_shape_and_values"
# subject = "zlib.ZLIB_VERSION"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.ZLIB_VERSION: ZLIB_VERSION and ZLIB_RUNTIME_VERSION are non-empty strings sharing a major version, and the named method/strategy/flush constants (DEFLATED, MAX_WBITS, Z_BEST_COMPRESSION, Z_FILTERED, Z_SYNC_FLUSH, DEF_BUF_SIZE) hold their documented int values"""
import zlib

# Version strings are present, non-empty, and share a major version. Exact
# numbers vary by build, so only structural shape is asserted.
assert isinstance(zlib.ZLIB_VERSION, str), "ZLIB_VERSION is str"
assert isinstance(zlib.ZLIB_RUNTIME_VERSION, str), "ZLIB_RUNTIME_VERSION is str"
assert len(zlib.ZLIB_VERSION) > 0, "ZLIB_VERSION non-empty"
assert zlib.ZLIB_RUNTIME_VERSION[0] == zlib.ZLIB_VERSION[0], "major version matches"

# Named method/strategy/flush constants are ints with documented values.
assert zlib.DEFLATED == 8, "DEFLATED == 8"
assert zlib.MAX_WBITS == 15, "MAX_WBITS == 15"
assert zlib.Z_BEST_COMPRESSION == 9, "Z_BEST_COMPRESSION == 9"
assert zlib.Z_FILTERED == 1, "Z_FILTERED == 1"
assert zlib.Z_SYNC_FLUSH == 2, "Z_SYNC_FLUSH == 2"
assert isinstance(zlib.DEF_BUF_SIZE, int) and zlib.DEF_BUF_SIZE > 0, "DEF_BUF_SIZE positive int"

print("version_constants_shape_and_values OK")

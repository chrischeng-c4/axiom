# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "native_int_size"
# subject = "struct.calcsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: native '@i' uses the platform int width so calcsize('@i') is 4 or 8 (and may carry native alignment), unlike the standard-size '=i'"""
import struct

# Native '@i' uses the platform int width.
_native = struct.calcsize("@i")
assert _native in (4, 8), f"native int size = {_native!r}"

# Standard-size '=i' is always exactly 4 bytes regardless of platform.
assert struct.calcsize("=i") == 4, "standard-size int is 4 bytes"

print("native_int_size OK")

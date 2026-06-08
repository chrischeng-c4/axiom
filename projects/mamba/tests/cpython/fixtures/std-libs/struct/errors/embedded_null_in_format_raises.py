# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "embedded_null_in_format_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
"""struct.calcsize: embedded_null_in_format_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("2\u0000i")
except struct.error:
    _raised = True
assert _raised, "embedded_null_in_format_raises: expected struct.error"
print("embedded_null_in_format_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "pack_value_out_of_range_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim truncates instead of raising (WI #3929; struct_mod.rs has no range check)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: pack_value_out_of_range_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("b", 1000)
except struct.error:
    _raised = True
assert _raised, "pack_value_out_of_range_raises: expected struct.error"
print("pack_value_out_of_range_raises OK")

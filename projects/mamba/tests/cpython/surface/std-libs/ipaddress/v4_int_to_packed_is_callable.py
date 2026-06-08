# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "v4_int_to_packed_is_callable"
# subject = "ipaddress.v4_int_to_packed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.v4_int_to_packed: v4_int_to_packed_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.v4_int_to_packed)
print("v4_int_to_packed_is_callable OK")

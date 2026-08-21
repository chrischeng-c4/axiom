# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "ipv6address_bad_colon_raises"
# subject = "ipaddress.IPv6Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Address: ipv6address_bad_colon_raises (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv6Address(":::1")
except ipaddress.AddressValueError:
    _raised = True
assert _raised, "ipv6address_bad_colon_raises: expected ipaddress.AddressValueError"
print("ipv6address_bad_colon_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "ipv4address_octet_over_255_raises"
# subject = "ipaddress.IPv4Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Address: ipv4address_octet_over_255_raises (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Address("256.0.0.1")
except ipaddress.AddressValueError:
    _raised = True
assert _raised, "ipv4address_octet_over_255_raises: expected ipaddress.AddressValueError"
print("ipv4address_octet_over_255_raises OK")

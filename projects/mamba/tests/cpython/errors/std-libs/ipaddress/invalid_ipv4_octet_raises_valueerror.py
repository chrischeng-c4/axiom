# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "invalid_ipv4_octet_raises_valueerror"
# subject = "ipaddress.ip_address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.ip_address: invalid_ipv4_octet_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.ip_address("999.0.0.1")
except ValueError:
    _raised = True
assert _raised, "invalid_ipv4_octet_raises_valueerror: expected ValueError"
print("invalid_ipv4_octet_raises_valueerror OK")

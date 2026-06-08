# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "supernet_of_cross_version_raises_typeerror"
# subject = "ipaddress.IPv6Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Network: supernet_of_cross_version_raises_typeerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv6Network("::1/128").supernet_of(ipaddress.IPv4Network("10.0.0.0/30"))
except TypeError:
    _raised = True
assert _raised, "supernet_of_cross_version_raises_typeerror: expected TypeError"
print("supernet_of_cross_version_raises_typeerror OK")

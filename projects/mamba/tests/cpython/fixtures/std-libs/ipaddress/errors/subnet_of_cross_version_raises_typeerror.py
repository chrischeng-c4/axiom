# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "subnet_of_cross_version_raises_typeerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: subnet_of_cross_version_raises_typeerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.0/30").subnet_of(ipaddress.IPv6Network("::1/128"))
except TypeError:
    _raised = True
assert _raised, "subnet_of_cross_version_raises_typeerror: expected TypeError"
print("subnet_of_cross_version_raises_typeerror OK")

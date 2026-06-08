# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "cross_version_network_compare_raises_typeerror"
# subject = "ipaddress.ip_network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.ip_network: cross_version_network_compare_raises_typeerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.ip_network("1.1.1.1/32") < ipaddress.ip_network("::1/128")
except TypeError:
    _raised = True
assert _raised, "cross_version_network_compare_raises_typeerror: expected TypeError"
print("cross_version_network_compare_raises_typeerror OK")

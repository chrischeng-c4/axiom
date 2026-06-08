# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "prefix_over_32_raises_netmaskvalueerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: prefix_over_32_raises_netmaskvalueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.0/40")
except ipaddress.NetmaskValueError:
    _raised = True
assert _raised, "prefix_over_32_raises_netmaskvalueerror: expected ipaddress.NetmaskValueError"
print("prefix_over_32_raises_netmaskvalueerror OK")

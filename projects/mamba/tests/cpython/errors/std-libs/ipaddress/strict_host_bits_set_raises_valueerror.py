# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "strict_host_bits_set_raises_valueerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: strict_host_bits_set_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.1/24", strict=True)
except ValueError:
    _raised = True
assert _raised, "strict_host_bits_set_raises_valueerror: expected ValueError"
print("strict_host_bits_set_raises_valueerror OK")

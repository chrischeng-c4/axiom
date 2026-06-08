# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "bad_ip_string_raises_valueerror"
# subject = "ipaddress.ip_address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.ip_address: bad_ip_string_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.ip_address("not.an.ip")
except ValueError:
    _raised = True
assert _raised, "bad_ip_string_raises_valueerror: expected ValueError"
print("bad_ip_string_raises_valueerror OK")

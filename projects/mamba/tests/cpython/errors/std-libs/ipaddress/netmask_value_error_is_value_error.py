# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "netmask_value_error_is_value_error"
# subject = "ipaddress.NetmaskValueError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.NetmaskValueError: NetmaskValueError is a subclass of ValueError so a plain `except ValueError` catches it"""
import ipaddress

assert issubclass(ipaddress.NetmaskValueError, ValueError), "NetmaskValueError <: ValueError"
_caught = False
try:
    ipaddress.IPv4Network("10.0.0.0/40")
except ValueError as e:
    _caught = type(e).__name__ == "NetmaskValueError"
assert _caught, "plain except ValueError catches NetmaskValueError"
print("netmask_value_error_is_value_error OK")

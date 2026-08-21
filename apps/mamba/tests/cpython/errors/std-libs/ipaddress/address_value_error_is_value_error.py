# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "address_value_error_is_value_error"
# subject = "ipaddress.AddressValueError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.AddressValueError: AddressValueError is a subclass of ValueError so a plain `except ValueError` catches it"""
import ipaddress

assert issubclass(ipaddress.AddressValueError, ValueError), "AddressValueError <: ValueError"
_caught = False
try:
    ipaddress.IPv4Address("256.0.0.1")
except ValueError as e:
    _caught = type(e).__name__ == "AddressValueError"
assert _caught, "plain except ValueError catches AddressValueError"
print("address_value_error_is_value_error OK")

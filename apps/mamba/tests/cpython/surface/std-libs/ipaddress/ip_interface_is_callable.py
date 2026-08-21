# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ip_interface_is_callable"
# subject = "ipaddress.ip_interface"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_interface: ip_interface_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.ip_interface)
print("ip_interface_is_callable OK")

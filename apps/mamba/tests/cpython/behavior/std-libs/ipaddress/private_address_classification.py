# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "private_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an RFC1918 address (192.168.1.1) is_private and not is_global"""
import ipaddress

a = ipaddress.ip_address("192.168.1.1")
assert a.is_private, "is_private"
assert not a.is_global, "not is_global"
print("private_address_classification OK")

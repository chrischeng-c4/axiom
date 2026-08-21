# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "global_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: a public address (8.8.8.8) is_global and not is_private"""
import ipaddress

a = ipaddress.ip_address("8.8.8.8")
assert a.is_global, "is_global"
assert not a.is_private, "not is_private"
print("global_address_classification OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "loopback_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: 127.0.0.1 is_loopback while 8.8.8.8 is not"""
import ipaddress

assert ipaddress.ip_address("127.0.0.1").is_loopback, "loopback"
assert not ipaddress.ip_address("8.8.8.8").is_loopback, "not loopback"
print("loopback_address_classification OK")

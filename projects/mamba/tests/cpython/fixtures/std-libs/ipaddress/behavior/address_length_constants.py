# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "address_length_constants"
# subject = "ipaddress.IPV4LENGTH"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPV4LENGTH: the IPV4LENGTH / IPV6LENGTH bit-width constants are 32 and 128"""
import ipaddress

assert ipaddress.IPV4LENGTH == 32, ipaddress.IPV4LENGTH
assert ipaddress.IPV6LENGTH == 128, ipaddress.IPV6LENGTH
print("address_length_constants OK")

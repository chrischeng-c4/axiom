# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_from_str_int_bytes_host_route"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: IPv4Network/IPv6Network accept str, int, and bytes and default to a host route (/32, /128) with no mask"""
import ipaddress

for addr in ("1.2.3.4", 16909060, b"\x01\x02\x03\x04"):
    n4 = ipaddress.IPv4Network(addr)
    assert str(n4) == "1.2.3.4/32", (type(addr).__name__, str(n4))
for addr in ("::1", 1, b"\x00" * 15 + b"\x01"):
    n6 = ipaddress.IPv6Network(addr)
    assert str(n6) == "::1/128", (type(addr).__name__, str(n6))
print("network_from_str_int_bytes_host_route OK")

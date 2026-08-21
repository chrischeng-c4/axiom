# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "gethostbyname_literal_ipv4_is_identity"
# subject = "socket.gethostbyname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.gethostbyname: a literal IPv4 address resolves to itself with no DNS round-trip (127.0.0.1, 10.0.0.1, 255.255.255.255)"""
import socket

for _addr in ("127.0.0.1", "10.0.0.1", "255.255.255.255"):
    assert socket.gethostbyname(_addr) == _addr, f"{_addr} should resolve to itself"
print("gethostbyname_literal_ipv4_is_identity OK")

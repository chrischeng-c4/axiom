# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getaddrinfo_parses_numeric_ipv6_and_scope"
# subject = "socket.getaddrinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: getaddrinfo lower-cases a numeric IPv6 literal and threads a %scope id into the sockaddr's scope-id field"""
import socket

(*_, sockaddr), = socket.getaddrinfo(
    "ff02::1de:c0:face:8D", 1234,
    socket.AF_INET6, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
assert sockaddr == ("ff02::1de:c0:face:8d", 1234, 0, 0), sockaddr

(*_, scoped), = socket.getaddrinfo(
    "ff02::1de:c0:face:8D%42", 1234,
    socket.AF_INET6, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
assert scoped == ("ff02::1de:c0:face:8d", 1234, 0, 42), scoped
print("getaddrinfo_parses_numeric_ipv6_and_scope OK")

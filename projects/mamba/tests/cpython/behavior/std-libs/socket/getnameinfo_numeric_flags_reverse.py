# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getnameinfo_numeric_flags_reverse"
# subject = "socket.getnameinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getnameinfo: getnameinfo with NI_NUMERICHOST|NI_NUMERICSERV reverses a numeric IPv6 address/port without DNS, lower-casing the host"""
import socket

ni = socket.getnameinfo(
    ("ff02::1de:c0:face:8D", 1234, 0, 0),
    socket.NI_NUMERICHOST | socket.NI_NUMERICSERV)
assert ni == ("ff02::1de:c0:face:8d", "1234"), ni
print("getnameinfo_numeric_flags_reverse OK")

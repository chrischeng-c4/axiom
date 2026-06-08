# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "ipproto_udp_attr"
# subject = "socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: ipproto_udp_attr (surface)."""
import socket

assert hasattr(socket, "IPPROTO_UDP")
print("ipproto_udp_attr OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "inet_aton_ntoa_roundtrip"
# subject = "socket.inet_aton"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_aton: inet_ntoa(inet_aton(ip)) is the identity for several dotted-quad IPv4 addresses including 255.255.255.255"""
import socket

for _ip in ("192.168.0.1", "10.0.0.1", "172.16.0.1", "255.255.255.255"):
    assert socket.inet_ntoa(socket.inet_aton(_ip)) == _ip, f"round-trip {_ip}"
print("inet_aton_ntoa_roundtrip OK")

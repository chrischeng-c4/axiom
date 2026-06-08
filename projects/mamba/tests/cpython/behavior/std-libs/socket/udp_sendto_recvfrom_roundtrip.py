# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "udp_sendto_recvfrom_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a connectionless UDP socket pair on loopback delivers a datagram: sendto(b'hello udp') is read back verbatim by recvfrom on the bound server"""
import socket

_udp_srv = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
_udp_srv.bind(("127.0.0.1", 0))
_udp_port = _udp_srv.getsockname()[1]
_udp_srv.settimeout(2.0)

_udp_cli = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
_udp_cli.sendto(b"hello udp", ("127.0.0.1", _udp_port))
_udp_data, _udp_addr = _udp_srv.recvfrom(1024)
assert _udp_data == b"hello udp", f"UDP data: {_udp_data!r}"
_udp_srv.close()
_udp_cli.close()
print("udp_sendto_recvfrom_roundtrip OK")

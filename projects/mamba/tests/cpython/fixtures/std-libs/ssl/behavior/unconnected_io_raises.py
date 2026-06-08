# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "unconnected_io_raises"
# subject = "ssl.SSLSocket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLSocket: I/O on an unconnected wrapped socket raises before any handshake: recv/recv_into/recvfrom/recvfrom_into/send/sendto raise OSError, while dup/sendmsg/recvmsg/recvmsg_into raise NotImplementedError"""
import ssl

import socket

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
_s = socket.socket(socket.AF_INET)
with _ctx.wrap_socket(_s, server_hostname="localhost") as _ss:
    for _name, _fn in (
        ("recv", lambda: _ss.recv(1)),
        ("recv_into", lambda: _ss.recv_into(bytearray(b"x"))),
        ("recvfrom", lambda: _ss.recvfrom(1)),
        ("recvfrom_into", lambda: _ss.recvfrom_into(bytearray(b"x"), 1)),
        ("send", lambda: _ss.send(b"x")),
        ("sendto", lambda: _ss.sendto(b"x", ("0.0.0.0", 0))),
    ):
        try:
            _fn()
            raise AssertionError(f"{_name} on unconnected should raise")
        except OSError:
            pass
    for _name, _fn in (
        ("dup", lambda: _ss.dup()),
        ("sendmsg", lambda: _ss.sendmsg([b"x"], (), 0, ("0.0.0.0", 0))),
        ("recvmsg", lambda: _ss.recvmsg(100)),
        ("recvmsg_into", lambda: _ss.recvmsg_into([bytearray(100)])),
    ):
        try:
            _fn()
            raise AssertionError(f"{_name} should be NotImplementedError")
        except NotImplementedError:
            pass

print("unconnected_io_raises OK")

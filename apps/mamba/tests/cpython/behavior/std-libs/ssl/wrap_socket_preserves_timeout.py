# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "wrap_socket_preserves_timeout"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: wrap_socket preserves the underlying socket's timeout: gettimeout() on the wrapped socket round-trips None, 0.0, and 5.0"""
import ssl

import socket


def _wrap(sock):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE
    return ctx.wrap_socket(sock, server_hostname="localhost")


for _t in (None, 0.0, 5.0):
    _s = socket.socket(socket.AF_INET)
    _s.settimeout(_t)
    with _wrap(_s) as _ss:
        assert _ss.gettimeout() == _t, f"timeout passthrough {_t!r}"

print("wrap_socket_preserves_timeout OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "server_side_with_hostname_raises"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: a server-side wrap_socket given a server_hostname raises ValueError (hostname only makes sense client-side)"""
import ssl

import socket

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
try:
    with socket.socket() as _sock:
        _srv.wrap_socket(_sock, True, server_hostname="some.hostname")
    raise AssertionError("server_side + server_hostname should raise")
except ValueError:
    pass

print("server_side_with_hostname_raises OK")

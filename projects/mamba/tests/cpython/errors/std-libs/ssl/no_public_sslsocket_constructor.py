# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "no_public_sslsocket_constructor"
# subject = "ssl.SSLSocket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLSocket: SSLSocket has no public constructor: ssl.SSLSocket(sock) raises TypeError naming the missing public constructor; the type is built only via wrap_socket"""
import ssl

import socket

try:
    with socket.socket() as _s:
        ssl.SSLSocket(_s)
    raise AssertionError("SSLSocket() should raise")
except TypeError as _e:
    assert "public constructor" in str(_e), f"SSLSocket msg: {_e}"

print("no_public_sslsocket_constructor OK")

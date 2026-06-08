# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "custom_sslsocket_object_classes_honored"
# subject = "ssl.SSLContext.sslsocket_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.sslsocket_class: a context's sslsocket_class / sslobject_class overrides are honored: wrap_socket returns the custom SSLSocket subclass and wrap_bio returns the custom SSLObject subclass"""
import ssl

import socket


class _MySSLSocket(ssl.SSLSocket):
    pass


class _MySSLObject(ssl.SSLObject):
    pass


_cc = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
_cc.sslsocket_class = _MySSLSocket
_cc.sslobject_class = _MySSLObject
with _cc.wrap_socket(socket.socket(), server_side=True) as _sock:
    assert isinstance(_sock, _MySSLSocket), "custom sslsocket_class honored"
_obj = _cc.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(), server_side=True)
assert isinstance(_obj, _MySSLObject), "custom sslobject_class honored"

print("custom_sslsocket_object_classes_honored OK")

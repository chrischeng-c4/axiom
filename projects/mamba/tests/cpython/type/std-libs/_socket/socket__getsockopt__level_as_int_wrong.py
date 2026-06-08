# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_socket"
# dimension = "type"
# case = "socket__getsockopt__level_as_int_wrong"
# subject = "_socket.socket.getsockopt(level: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _socket.socket.getsockopt(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _socket import socket
obj = object.__new__(socket)
try:
    obj.getsockopt("not_an_int", 0)  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

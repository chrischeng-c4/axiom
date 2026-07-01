# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_socket"
# dimension = "type"
# case = "socket__share__process_id_as_int_wrong"
# subject = "_socket.socket.share(process_id: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _socket.socket.share(process_id: int); call it with the wrong type.

typeshed contract: process_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _socket import socket
obj = object.__new__(socket)
try:
    obj.share("not_an_int")  # process_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_socket"
# dimension = "type"
# case = "socket__connect_ex__address_as__Address_wrong"
# subject = "_socket.socket.connect_ex(address: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _socket.socket.connect_ex(address: _Address); call it with the wrong type.

typeshed contract: address is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _socket import socket
obj = object.__new__(socket)
try:
    obj.connect_ex(_W())  # address: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

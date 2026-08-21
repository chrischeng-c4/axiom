# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "type"
# case = "SocketIO__write__b_as_ReadableBuffer_wrong"
# subject = "socket.SocketIO.write(b: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socket.SocketIO.write(b: ReadableBuffer); call it with the wrong type.

typeshed contract: b is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socket import SocketIO
obj = object.__new__(SocketIO)
try:
    obj.write(_W())  # b: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

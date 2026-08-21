# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__recv_into__buffer_as_WriteableBuffer_wrong"
# subject = "ssl.SSLSocket.recv_into(buffer: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.recv_into(buffer: WriteableBuffer); call it with the wrong type.

typeshed contract: buffer is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.recv_into(_W())  # buffer: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

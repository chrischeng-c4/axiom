# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_socket"
# dimension = "type"
# case = "CMSG_LEN__length_as_int_wrong"
# subject = "_socket.CMSG_LEN(length: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _socket.CMSG_LEN(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _socket import CMSG_LEN
try:
    CMSG_LEN("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

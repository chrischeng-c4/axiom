# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "type"
# case = "socketpair__family_as_int_wrong"
# subject = "socket.socketpair(family: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed family"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socket.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed family
# mamba-strict-type: TypeError
"""Type wall: socket.socketpair(family: int); call it with the wrong type.

typeshed contract: family is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from socket import socketpair
try:
    socketpair("not_an_int")  # family: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

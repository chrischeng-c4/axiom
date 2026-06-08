# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "type"
# case = "socket__set_inheritable__inheritable_as_bool_wrong"
# subject = "socket.socket.set_inheritable(inheritable: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed inheritable"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socket.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed inheritable
# mamba-strict-type: TypeError
"""Type wall: socket.socket.set_inheritable(inheritable: bool); call it with the wrong type.

typeshed contract: inheritable is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from socket import socket
obj = object.__new__(socket)
try:
    obj.set_inheritable("not_a_bool")  # inheritable: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

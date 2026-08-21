# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext____new____protocol_as_int_wrong"
# subject = "ssl.SSLContext.__new__(protocol: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed protocol"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed protocol
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.__new__(protocol: int); call it with the wrong type.

typeshed contract: protocol is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.__new__("not_an_int")  # protocol: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

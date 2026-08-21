# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__serve_forever__poll_interval_as_float_wrong"
# subject = "socketserver.BaseServer.serve_forever(poll_interval: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.serve_forever(poll_interval: float); call it with the wrong type.

typeshed contract: poll_interval is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.serve_forever("not_a_float")  # poll_interval: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SocketHandler__send__s_as_ReadableBuffer_wrong"
# subject = "logging.handlers.SocketHandler.send(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SocketHandler.send(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SocketHandler
obj = object.__new__(SocketHandler)
try:
    obj.send(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_streams"
# dimension = "type"
# case = "StreamReader__set_exception__exc_as_Exception_wrong"
# subject = "asyncio.streams.StreamReader.set_exception(exc: Exception)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.streams.StreamReader.set_exception(exc: Exception); call it with the wrong type.

typeshed contract: exc is Exception. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.streams import StreamReader
obj = object.__new__(StreamReader)
try:
    obj.set_exception(_W())  # exc: Exception <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

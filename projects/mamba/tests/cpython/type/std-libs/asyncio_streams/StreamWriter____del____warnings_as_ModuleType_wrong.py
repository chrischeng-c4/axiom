# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_streams"
# dimension = "type"
# case = "StreamWriter____del____warnings_as_ModuleType_wrong"
# subject = "asyncio.streams.StreamWriter.__del__(warnings: ModuleType)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed warnings"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/streams.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed warnings
# mamba-strict-type: TypeError
"""Type wall: asyncio.streams.StreamWriter.__del__(warnings: ModuleType); call it with the wrong type.

typeshed contract: warnings is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.streams import StreamWriter
obj = object.__new__(StreamWriter)
try:
    obj.__del__(_W())  # warnings: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

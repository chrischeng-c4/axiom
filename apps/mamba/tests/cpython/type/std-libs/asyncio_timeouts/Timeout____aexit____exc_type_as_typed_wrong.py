# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "Timeout____aexit____exc_type_as_typed_wrong"
# subject = "asyncio.timeouts.Timeout.__aexit__(exc_type: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.Timeout.__aexit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import Timeout
obj = object.__new__(Timeout)
try:
    obj.__aexit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

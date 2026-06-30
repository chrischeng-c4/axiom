# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "BaseTransport__init__extra_as_typed_wrong"
# subject = "asyncio.transports.BaseTransport.__init__(extra: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.BaseTransport.__init__(extra: typed); call it with the wrong type.

typeshed contract: extra is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import BaseTransport
try:
    BaseTransport(_W())  # extra: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

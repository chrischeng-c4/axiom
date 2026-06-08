# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_trsock"
# dimension = "type"
# case = "TransportSocket__share__process_id_as_int_wrong"
# subject = "asyncio.trsock.TransportSocket.share(process_id: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed process_id"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/trsock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed process_id
# mamba-strict-type: TypeError
"""Type wall: asyncio.trsock.TransportSocket.share(process_id: int); call it with the wrong type.

typeshed contract: process_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.trsock import TransportSocket
obj = object.__new__(TransportSocket)
try:
    obj.share("not_an_int")  # process_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_trsock"
# dimension = "type"
# case = "TransportSocket__sendmsg_afalg__msg_as_Iterable_wrong"
# subject = "asyncio.trsock.TransportSocket.sendmsg_afalg(msg: Iterable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/trsock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg
# mamba-strict-type: TypeError
"""Type wall: asyncio.trsock.TransportSocket.sendmsg_afalg(msg: Iterable); call it with the wrong type.

typeshed contract: msg is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.trsock import TransportSocket
obj = object.__new__(TransportSocket)
try:
    obj.sendmsg_afalg(_W())  # msg: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

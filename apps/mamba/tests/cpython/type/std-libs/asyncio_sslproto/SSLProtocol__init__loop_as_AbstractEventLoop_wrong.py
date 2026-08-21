# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "SSLProtocol__init__loop_as_AbstractEventLoop_wrong"
# subject = "asyncio.sslproto.SSLProtocol.__init__(loop: AbstractEventLoop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.SSLProtocol.__init__(loop: AbstractEventLoop); call it with the wrong type.

typeshed contract: loop is AbstractEventLoop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.sslproto import SSLProtocol
try:
    SSLProtocol(_W(), None, None, None)  # loop: AbstractEventLoop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

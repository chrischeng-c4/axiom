# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_streams"
# dimension = "type"
# case = "StreamWriter__start_tls__sslcontext_as_SSLContext_wrong"
# subject = "asyncio.streams.StreamWriter.start_tls(sslcontext: SSLContext)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.streams.StreamWriter.start_tls(sslcontext: SSLContext); call it with the wrong type.

typeshed contract: sslcontext is SSLContext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.streams import StreamWriter
obj = object.__new__(StreamWriter)
try:
    obj.start_tls(_W())  # sslcontext: SSLContext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

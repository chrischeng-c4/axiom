# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_streams"
# dimension = "type"
# case = "start_server__client_connected_cb_as__ClientConnectedCallback_wrong"
# subject = "asyncio.streams.start_server(client_connected_cb: _ClientConnectedCallback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.streams.start_server(client_connected_cb: _ClientConnectedCallback); call it with the wrong type.

typeshed contract: client_connected_cb is _ClientConnectedCallback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.streams import start_server
try:
    start_server(_W())  # client_connected_cb: _ClientConnectedCallback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_graph"
# dimension = "type"
# case = "capture_call_graph__future_as_typed_wrong"
# subject = "asyncio.graph.capture_call_graph(future: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed future"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/graph.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed future
# mamba-strict-type: TypeError
"""Type wall: asyncio.graph.capture_call_graph(future: typed); call it with the wrong type.

typeshed contract: future is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.graph import capture_call_graph
try:
    capture_call_graph(_W())  # future: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "getasyncgenstate__agen_as_AsyncGenerator_wrong"
# subject = "inspect.getasyncgenstate(agen: AsyncGenerator)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed agen"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed agen
# mamba-strict-type: TypeError
"""Type wall: inspect.getasyncgenstate(agen: AsyncGenerator); call it with the wrong type.

typeshed contract: agen is AsyncGenerator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from inspect import getasyncgenstate
try:
    getasyncgenstate(_W())  # agen: AsyncGenerator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "getasyncgenlocals__agen_as_AsyncGeneratorType_wrong"
# subject = "inspect.getasyncgenlocals(agen: AsyncGeneratorType)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed agen"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed agen
# mamba-strict-type: TypeError
"""Type wall: inspect.getasyncgenlocals(agen: AsyncGeneratorType); call it with the wrong type.

typeshed contract: agen is AsyncGeneratorType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from inspect import getasyncgenlocals
try:
    getasyncgenlocals(_W())  # agen: AsyncGeneratorType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

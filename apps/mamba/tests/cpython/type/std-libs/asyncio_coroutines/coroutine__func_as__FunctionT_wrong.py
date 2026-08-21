# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_coroutines"
# dimension = "type"
# case = "coroutine__func_as__FunctionT_wrong"
# subject = "asyncio.coroutines.coroutine(func: _FunctionT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/coroutines.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.coroutines.coroutine(func: _FunctionT); call it with the wrong type.

typeshed contract: func is _FunctionT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.coroutines import coroutine
try:
    coroutine(_W())  # func: _FunctionT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

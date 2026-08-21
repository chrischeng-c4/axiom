# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "Future__set_result__result_as__T_wrong"
# subject = "concurrent.futures._base.Future.set_result(result: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed result"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed result
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.Future.set_result(result: _T); call it with the wrong type.

typeshed contract: result is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import Future
obj = object.__new__(Future)
try:
    obj.set_result(_W())  # result: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

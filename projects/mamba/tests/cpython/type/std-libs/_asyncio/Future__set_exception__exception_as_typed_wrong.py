# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "type"
# case = "Future__set_exception__exception_as_typed_wrong"
# subject = "_asyncio.Future.set_exception(exception: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exception"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_asyncio.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exception
# mamba-strict-type: TypeError
"""Type wall: _asyncio.Future.set_exception(exception: typed); call it with the wrong type.

typeshed contract: exception is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _asyncio import Future
obj = object.__new__(Future)
try:
    obj.set_exception(_W())  # exception: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

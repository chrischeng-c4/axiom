# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "Coroutine__send__value_as__SendT_nd_contra_wrong"
# subject = "typing.Coroutine.send(value: _SendT_nd_contra)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: typing.Coroutine.send(value: _SendT_nd_contra); call it with the wrong type.

typeshed contract: value is _SendT_nd_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import Coroutine
obj = object.__new__(Coroutine)
try:
    obj.send(_W())  # value: _SendT_nd_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

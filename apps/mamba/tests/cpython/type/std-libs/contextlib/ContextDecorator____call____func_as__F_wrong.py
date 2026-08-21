# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "ContextDecorator____call____func_as__F_wrong"
# subject = "contextlib.ContextDecorator.__call__(func: _F)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed func"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed func
# mamba-strict-type: TypeError
"""Type wall: contextlib.ContextDecorator.__call__(func: _F); call it with the wrong type.

typeshed contract: func is _F. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import ContextDecorator
obj = object.__new__(ContextDecorator)
try:
    obj.__call__(_W())  # func: _F <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

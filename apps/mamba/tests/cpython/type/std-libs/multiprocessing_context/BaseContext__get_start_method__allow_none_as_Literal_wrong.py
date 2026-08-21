# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "BaseContext__get_start_method__allow_none_as_Literal_wrong"
# subject = "multiprocessing.context.BaseContext.get_start_method(allow_none: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.BaseContext.get_start_method(allow_none: Literal); call it with the wrong type.

typeshed contract: allow_none is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.context import BaseContext
obj = object.__new__(BaseContext)
try:
    obj.get_start_method(_W())  # allow_none: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

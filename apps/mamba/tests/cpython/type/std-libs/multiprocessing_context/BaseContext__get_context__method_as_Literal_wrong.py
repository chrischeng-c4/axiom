# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "BaseContext__get_context__method_as_Literal_wrong"
# subject = "multiprocessing.context.BaseContext.get_context(method: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.BaseContext.get_context(method: Literal); call it with the wrong type.

typeshed contract: method is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.context import BaseContext
obj = object.__new__(BaseContext)
try:
    obj.get_context(_W())  # method: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

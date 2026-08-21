# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "DefaultContext__get_start_method__allow_none_as_bool_wrong"
# subject = "multiprocessing.context.DefaultContext.get_start_method(allow_none: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.DefaultContext.get_start_method(allow_none: bool); call it with the wrong type.

typeshed contract: allow_none is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.context import DefaultContext
obj = object.__new__(DefaultContext)
try:
    obj.get_start_method("not_a_bool")  # allow_none: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

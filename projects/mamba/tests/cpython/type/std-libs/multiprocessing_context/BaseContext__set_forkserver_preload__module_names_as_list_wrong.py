# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "BaseContext__set_forkserver_preload__module_names_as_list_wrong"
# subject = "multiprocessing.context.BaseContext.set_forkserver_preload(module_names: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed module_names"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed module_names
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.BaseContext.set_forkserver_preload(module_names: list); call it with the wrong type.

typeshed contract: module_names is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.context import BaseContext
obj = object.__new__(BaseContext)
try:
    obj.set_forkserver_preload(12345)  # module_names: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

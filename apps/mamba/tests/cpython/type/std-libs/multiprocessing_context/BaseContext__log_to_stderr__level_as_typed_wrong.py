# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "BaseContext__log_to_stderr__level_as_typed_wrong"
# subject = "multiprocessing.context.BaseContext.log_to_stderr(level: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.BaseContext.log_to_stderr(level: typed); call it with the wrong type.

typeshed contract: level is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.context import BaseContext
obj = object.__new__(BaseContext)
try:
    obj.log_to_stderr(_W())  # level: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

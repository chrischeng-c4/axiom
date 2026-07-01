# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_context"
# dimension = "type"
# case = "ForkProcess___Popen__process_obj_as_BaseProcess_wrong"
# subject = "multiprocessing.context.ForkProcess._Popen(process_obj: BaseProcess)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/context.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.context.ForkProcess._Popen(process_obj: BaseProcess); call it with the wrong type.

typeshed contract: process_obj is BaseProcess. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.context import ForkProcess
try:
    ForkProcess._Popen(_W())  # process_obj: BaseProcess <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

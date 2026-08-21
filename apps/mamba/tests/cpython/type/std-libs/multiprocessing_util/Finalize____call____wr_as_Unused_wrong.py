# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_util"
# dimension = "type"
# case = "Finalize____call____wr_as_Unused_wrong"
# subject = "multiprocessing.util.Finalize.__call__(wr: Unused)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.util.Finalize.__call__(wr: Unused); call it with the wrong type.

typeshed contract: wr is Unused. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.util import Finalize
obj = object.__new__(Finalize)
try:
    obj.__call__(_W())  # wr: Unused <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

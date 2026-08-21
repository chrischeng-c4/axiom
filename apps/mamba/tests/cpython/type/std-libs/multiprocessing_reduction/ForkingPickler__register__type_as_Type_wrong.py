# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_reduction"
# dimension = "type"
# case = "ForkingPickler__register__type_as_Type_wrong"
# subject = "multiprocessing.reduction.ForkingPickler.register(type: Type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/reduction.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.reduction.ForkingPickler.register(type: Type); call it with the wrong type.

typeshed contract: type is Type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.reduction import ForkingPickler
try:
    ForkingPickler.register(_W(), None)  # type: Type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

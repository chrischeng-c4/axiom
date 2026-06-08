# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_synchronize"
# dimension = "type"
# case = "Condition__wait_for__predicate_as_Callable_wrong"
# subject = "multiprocessing.synchronize.Condition.wait_for(predicate: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/synchronize.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.synchronize.Condition.wait_for(predicate: Callable); call it with the wrong type.

typeshed contract: predicate is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.synchronize import Condition
obj = object.__new__(Condition)
try:
    obj.wait_for(_W())  # predicate: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

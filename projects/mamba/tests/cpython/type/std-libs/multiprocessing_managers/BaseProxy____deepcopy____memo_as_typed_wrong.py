# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_managers"
# dimension = "type"
# case = "BaseProxy____deepcopy____memo_as_typed_wrong"
# subject = "multiprocessing.managers.BaseProxy.__deepcopy__(memo: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed memo"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/managers.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed memo
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.managers.BaseProxy.__deepcopy__(memo: typed); call it with the wrong type.

typeshed contract: memo is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.managers import BaseProxy
obj = object.__new__(BaseProxy)
try:
    obj.__deepcopy__(_W())  # memo: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

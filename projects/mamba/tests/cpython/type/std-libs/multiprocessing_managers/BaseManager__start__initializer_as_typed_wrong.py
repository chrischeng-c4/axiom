# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_managers"
# dimension = "type"
# case = "BaseManager__start__initializer_as_typed_wrong"
# subject = "multiprocessing.managers.BaseManager.start(initializer: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed initializer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/managers.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed initializer
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.managers.BaseManager.start(initializer: typed); call it with the wrong type.

typeshed contract: initializer is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.managers import BaseManager
obj = object.__new__(BaseManager)
try:
    obj.start(_W())  # initializer: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

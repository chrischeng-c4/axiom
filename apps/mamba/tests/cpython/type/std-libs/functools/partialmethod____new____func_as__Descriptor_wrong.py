# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "type"
# case = "partialmethod____new____func_as__Descriptor_wrong"
# subject = "functools.partialmethod.__new__(func: _Descriptor)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed func"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/functools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed func
# mamba-strict-type: TypeError
"""Type wall: functools.partialmethod.__new__(func: _Descriptor); call it with the wrong type.

typeshed contract: func is _Descriptor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from functools import partialmethod
obj = object.__new__(partialmethod)
try:
    obj.__new__(_W())  # func: _Descriptor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

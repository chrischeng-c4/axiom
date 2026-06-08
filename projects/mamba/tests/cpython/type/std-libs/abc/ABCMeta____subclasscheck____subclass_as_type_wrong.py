# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "type"
# case = "ABCMeta____subclasscheck____subclass_as_type_wrong"
# subject = "abc.ABCMeta.__subclasscheck__(subclass: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subclass"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/abc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subclass
# mamba-strict-type: TypeError
"""Type wall: abc.ABCMeta.__subclasscheck__(subclass: type); call it with the wrong type.

typeshed contract: subclass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from abc import ABCMeta
obj = object.__new__(ABCMeta)
try:
    obj.__subclasscheck__(_W())  # subclass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

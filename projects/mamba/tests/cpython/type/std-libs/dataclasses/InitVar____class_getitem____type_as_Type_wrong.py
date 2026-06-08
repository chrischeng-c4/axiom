# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "type"
# case = "InitVar____class_getitem____type_as_Type_wrong"
# subject = "dataclasses.InitVar.__class_getitem__(type: Type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dataclasses.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type
# mamba-strict-type: TypeError
"""Type wall: dataclasses.InitVar.__class_getitem__(type: Type); call it with the wrong type.

typeshed contract: type is Type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dataclasses import InitVar
obj = object.__new__(InitVar)
try:
    obj.__class_getitem__(_W())  # type: Type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

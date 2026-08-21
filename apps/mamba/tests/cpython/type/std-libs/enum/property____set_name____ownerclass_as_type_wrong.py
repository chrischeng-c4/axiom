# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "property____set_name____ownerclass_as_type_wrong"
# subject = "enum.property.__set_name__(ownerclass: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ownerclass"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ownerclass
# mamba-strict-type: TypeError
"""Type wall: enum.property.__set_name__(ownerclass: type); call it with the wrong type.

typeshed contract: ownerclass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import property
obj = object.__new__(property)
try:
    obj.__set_name__(_W(), "")  # ownerclass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

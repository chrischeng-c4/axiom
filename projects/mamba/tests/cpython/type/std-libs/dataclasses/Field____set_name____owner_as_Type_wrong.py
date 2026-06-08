# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "type"
# case = "Field____set_name____owner_as_Type_wrong"
# subject = "dataclasses.Field.__set_name__(owner: Type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed owner"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dataclasses.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed owner
# mamba-strict-type: TypeError
"""Type wall: dataclasses.Field.__set_name__(owner: Type); call it with the wrong type.

typeshed contract: owner is Type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dataclasses import Field
obj = object.__new__(Field)
try:
    obj.__set_name__(_W(), "")  # owner: Type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

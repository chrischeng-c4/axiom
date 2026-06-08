# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____setattr____name_as_Unused_wrong"
# subject = "uuid.UUID.__setattr__(name: Unused)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed name
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__setattr__(name: Unused); call it with the wrong type.

typeshed contract: name is Unused. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__setattr__(_W(), None)  # name: Unused <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

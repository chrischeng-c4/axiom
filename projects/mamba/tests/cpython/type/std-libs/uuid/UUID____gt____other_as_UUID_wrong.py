# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____gt____other_as_UUID_wrong"
# subject = "uuid.UUID.__gt__(other: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__gt__(other: UUID); call it with the wrong type.

typeshed contract: other is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__gt__(_W())  # other: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

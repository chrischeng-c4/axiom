# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid3__namespace_as_UUID_wrong"
# subject = "uuid.uuid3(namespace: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid3(namespace: UUID); call it with the wrong type.

typeshed contract: namespace is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid3
try:
    uuid3(_W(), None)  # namespace: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

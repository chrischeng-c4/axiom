# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_thread"
# dimension = "type"
# case = "LockType____exit____type_as_typed_wrong"
# subject = "_thread.LockType.__exit__(type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _thread.LockType.__exit__(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _thread import LockType
obj = object.__new__(LockType)
try:
    obj.__exit__(_W(), None, None)  # type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

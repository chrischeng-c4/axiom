# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "type"
# case = "Condition____exit____exc_type_as_typed_wrong"
# subject = "threading.Condition.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/threading.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type
# mamba-strict-type: TypeError
"""Type wall: threading.Condition.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from threading import Condition
obj = object.__new__(Condition)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

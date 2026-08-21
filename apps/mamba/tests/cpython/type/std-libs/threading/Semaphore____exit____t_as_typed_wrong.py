# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "type"
# case = "Semaphore____exit____t_as_typed_wrong"
# subject = "threading.Semaphore.__exit__(t: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed t"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/threading.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed t
# mamba-strict-type: TypeError
"""Type wall: threading.Semaphore.__exit__(t: typed); call it with the wrong type.

typeshed contract: t is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from threading import Semaphore
obj = object.__new__(Semaphore)
try:
    obj.__exit__(_W(), None, None)  # t: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

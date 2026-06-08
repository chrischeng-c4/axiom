# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__autorange__callback_as_typed_wrong"
# subject = "timeit.Timer.autorange(callback: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callback"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callback
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.autorange(callback: typed); call it with the wrong type.

typeshed contract: callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.autorange(_W())  # callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

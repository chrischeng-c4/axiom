# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__timeit__number_as_int_wrong"
# subject = "timeit.Timer.timeit(number: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.timeit(number: int); call it with the wrong type.

typeshed contract: number is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.timeit("not_an_int")  # number: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

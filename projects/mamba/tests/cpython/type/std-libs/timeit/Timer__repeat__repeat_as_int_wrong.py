# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__repeat__repeat_as_int_wrong"
# subject = "timeit.Timer.repeat(repeat: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.repeat(repeat: int); call it with the wrong type.

typeshed contract: repeat is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.repeat("not_an_int")  # repeat: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

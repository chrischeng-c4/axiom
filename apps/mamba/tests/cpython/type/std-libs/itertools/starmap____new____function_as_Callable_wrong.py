# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "type"
# case = "starmap____new____function_as_Callable_wrong"
# subject = "itertools.starmap.__new__(function: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed function"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/itertools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed function
# mamba-strict-type: TypeError
"""Type wall: itertools.starmap.__new__(function: Callable); call it with the wrong type.

typeshed contract: function is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from itertools import starmap
obj = object.__new__(starmap)
try:
    obj.__new__(_W(), None)  # function: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

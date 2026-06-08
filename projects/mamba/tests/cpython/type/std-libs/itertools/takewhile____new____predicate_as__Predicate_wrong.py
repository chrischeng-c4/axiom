# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "type"
# case = "takewhile____new____predicate_as__Predicate_wrong"
# subject = "itertools.takewhile.__new__(predicate: _Predicate)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/itertools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate
# mamba-strict-type: TypeError
"""Type wall: itertools.takewhile.__new__(predicate: _Predicate); call it with the wrong type.

typeshed contract: predicate is _Predicate. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from itertools import takewhile
obj = object.__new__(takewhile)
try:
    obj.__new__(_W(), None)  # predicate: _Predicate <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

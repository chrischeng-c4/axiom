# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "type"
# case = "zip_longest____new____iter1_as_Iterable_wrong"
# subject = "itertools.zip_longest.__new__(iter1: Iterable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed iter1"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/itertools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed iter1
# mamba-strict-type: TypeError
"""Type wall: itertools.zip_longest.__new__(iter1: Iterable); call it with the wrong type.

typeshed contract: iter1 is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from itertools import zip_longest
obj = object.__new__(zip_longest)
try:
    obj.__new__(_W())  # iter1: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

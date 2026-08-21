# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "type"
# case = "compress____new____data_as_Iterable_wrong"
# subject = "itertools.compress.__new__(data: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/itertools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: itertools.compress.__new__(data: Iterable); call it with the wrong type.

typeshed contract: data is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from itertools import compress
obj = object.__new__(compress)
try:
    obj.__new__(_W(), None)  # data: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

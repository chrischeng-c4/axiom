# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "structseq____new____sequence_as_Iterable_wrong"
# subject = "_typeshed.structseq.__new__(sequence: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed.structseq.__new__(sequence: Iterable); call it with the wrong type.

typeshed contract: sequence is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _typeshed import structseq
obj = object.__new__(structseq)
try:
    obj.__new__(_W())  # sequence: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

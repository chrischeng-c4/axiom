# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____new____typecode_as__FloatTypeCode_wrong"
# subject = "array.array.__new__(typecode: _FloatTypeCode)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typecode"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typecode
# mamba-strict-type: TypeError
"""Type wall: array.array.__new__(typecode: _FloatTypeCode); call it with the wrong type.

typeshed contract: typecode is _FloatTypeCode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.__new__(_W())  # typecode: _FloatTypeCode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

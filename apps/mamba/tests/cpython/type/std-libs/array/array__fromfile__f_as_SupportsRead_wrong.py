# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__fromfile__f_as_SupportsRead_wrong"
# subject = "array.array.fromfile(f: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.fromfile(f: SupportsRead); call it with the wrong type.

typeshed contract: f is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.fromfile(_W(), 0)  # f: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

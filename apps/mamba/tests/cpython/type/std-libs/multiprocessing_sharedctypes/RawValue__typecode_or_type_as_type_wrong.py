# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawValue__typecode_or_type_as_type_wrong"
# subject = "multiprocessing.sharedctypes.RawValue(typecode_or_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawValue(typecode_or_type: type); call it with the wrong type.

typeshed contract: typecode_or_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import RawValue
try:
    RawValue(_W())  # typecode_or_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawValue__typecode_or_type_as_str_wrong"
# subject = "multiprocessing.sharedctypes.RawValue(typecode_or_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawValue(typecode_or_type: str); call it with the wrong type.

typeshed contract: typecode_or_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.sharedctypes import RawValue
try:
    RawValue(12345)  # typecode_or_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

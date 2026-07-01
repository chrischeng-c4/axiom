# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "HASHXOF__digest__length_as_int_wrong"
# subject = "_hashlib.HASHXOF.digest(length: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.HASHXOF.digest(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _hashlib import HASHXOF
obj = object.__new__(HASHXOF)
try:
    obj.digest("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

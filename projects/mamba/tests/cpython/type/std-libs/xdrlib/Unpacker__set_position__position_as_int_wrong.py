# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__set_position__position_as_int_wrong"
# subject = "xdrlib.Unpacker.set_position(position: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.set_position(position: int); call it with the wrong type.

typeshed contract: position is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.set_position("not_an_int")  # position: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

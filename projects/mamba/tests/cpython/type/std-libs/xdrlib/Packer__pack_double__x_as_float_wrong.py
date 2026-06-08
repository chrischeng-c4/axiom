# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_double__x_as_float_wrong"
# subject = "xdrlib.Packer.pack_double(x: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_double(x: float); call it with the wrong type.

typeshed contract: x is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_double("not_a_float")  # x: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

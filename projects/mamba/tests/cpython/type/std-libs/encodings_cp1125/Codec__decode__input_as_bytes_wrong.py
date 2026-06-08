# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_cp1125"
# dimension = "type"
# case = "Codec__decode__input_as_bytes_wrong"
# subject = "encodings.cp1125.Codec.decode(input: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed input"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/cp1125.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed input
# mamba-strict-type: TypeError
"""Type wall: encodings.cp1125.Codec.decode(input: bytes); call it with the wrong type.

typeshed contract: input is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.cp1125 import Codec
obj = object.__new__(Codec)
try:
    obj.decode(12345)  # input: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

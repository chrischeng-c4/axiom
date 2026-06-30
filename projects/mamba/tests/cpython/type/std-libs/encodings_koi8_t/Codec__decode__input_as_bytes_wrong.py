# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_koi8_t"
# dimension = "type"
# case = "Codec__decode__input_as_bytes_wrong"
# subject = "encodings.koi8_t.Codec.decode(input: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/koi8_t.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.koi8_t.Codec.decode(input: bytes); call it with the wrong type.

typeshed contract: input is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.koi8_t import Codec
obj = object.__new__(Codec)
try:
    obj.decode(12345)  # input: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

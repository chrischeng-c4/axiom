# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_mac_turkish"
# dimension = "type"
# case = "Codec__decode__input_as_bytes_wrong"
# subject = "encodings.mac_turkish.Codec.decode(input: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/mac_turkish.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.mac_turkish.Codec.decode(input: bytes); call it with the wrong type.

typeshed contract: input is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.mac_turkish import Codec
obj = object.__new__(Codec)
try:
    obj.decode(12345)  # input: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

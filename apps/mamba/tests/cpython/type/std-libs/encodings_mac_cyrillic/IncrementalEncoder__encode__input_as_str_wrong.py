# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_mac_cyrillic"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_str_wrong"
# subject = "encodings.mac_cyrillic.IncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/mac_cyrillic.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.mac_cyrillic.IncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.mac_cyrillic import IncrementalEncoder
obj = object.__new__(IncrementalEncoder)
try:
    obj.encode(12345)  # input: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_punycode"
# dimension = "type"
# case = "decode_generalized_number__extended_as_bytes_wrong"
# subject = "encodings.punycode.decode_generalized_number(extended: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/punycode.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.punycode.decode_generalized_number(extended: bytes); call it with the wrong type.

typeshed contract: extended is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.punycode import decode_generalized_number
try:
    decode_generalized_number(12345, 0, 0, "")  # extended: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

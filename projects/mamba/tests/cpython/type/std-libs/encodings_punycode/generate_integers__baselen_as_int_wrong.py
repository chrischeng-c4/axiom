# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_punycode"
# dimension = "type"
# case = "generate_integers__baselen_as_int_wrong"
# subject = "encodings.punycode.generate_integers(baselen: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/punycode.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.punycode.generate_integers(baselen: int); call it with the wrong type.

typeshed contract: baselen is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.punycode import generate_integers
try:
    generate_integers("not_an_int", None)  # baselen: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

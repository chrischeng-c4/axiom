# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_punycode"
# dimension = "type"
# case = "generate_generalized_integer__N_as_int_wrong"
# subject = "encodings.punycode.generate_generalized_integer(N: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/punycode.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.punycode.generate_generalized_integer(N: int); call it with the wrong type.

typeshed contract: N is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.punycode import generate_generalized_integer
try:
    generate_generalized_integer("not_an_int", 0)  # N: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_punycode"
# dimension = "type"
# case = "adapt__delta_as_int_wrong"
# subject = "encodings.punycode.adapt(delta: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/punycode.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.punycode.adapt(delta: int); call it with the wrong type.

typeshed contract: delta is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.punycode import adapt
try:
    adapt("not_an_int", True, 0)  # delta: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

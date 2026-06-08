# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "ToASCII__label_as_str_wrong"
# subject = "encodings.idna.ToASCII(label: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.ToASCII(label: str); call it with the wrong type.

typeshed contract: label is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.idna import ToASCII
try:
    ToASCII(12345)  # label: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

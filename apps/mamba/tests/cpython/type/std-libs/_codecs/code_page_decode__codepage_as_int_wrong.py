# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_codecs"
# dimension = "type"
# case = "code_page_decode__codepage_as_int_wrong"
# subject = "_codecs.code_page_decode(codepage: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _codecs.code_page_decode(codepage: int); call it with the wrong type.

typeshed contract: codepage is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _codecs import code_page_decode
try:
    code_page_decode("not_an_int", None)  # codepage: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

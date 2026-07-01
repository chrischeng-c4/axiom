# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_codecs"
# dimension = "type"
# case = "code_page_encode__code_page_as_int_wrong"
# subject = "_codecs.code_page_encode(code_page: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _codecs.code_page_encode(code_page: int); call it with the wrong type.

typeshed contract: code_page is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _codecs import code_page_encode
try:
    code_page_encode("not_an_int", "")  # code_page: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_parse"
# dimension = "type"
# case = "Tokenizer__seek__index_as_int_wrong"
# subject = "sre_parse.Tokenizer.seek(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sre_parse.Tokenizer.seek(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sre_parse import Tokenizer
obj = object.__new__(Tokenizer)
try:
    obj.seek("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

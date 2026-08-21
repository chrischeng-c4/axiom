# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__escape_brackets__token_as_str_wrong"
# subject = "tokenize.Untokenizer.escape_brackets(token: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.escape_brackets(token: str); call it with the wrong type.

typeshed contract: token is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.escape_brackets(12345)  # token: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

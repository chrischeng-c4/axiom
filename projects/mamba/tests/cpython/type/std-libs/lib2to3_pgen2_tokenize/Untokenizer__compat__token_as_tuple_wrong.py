# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_tokenize"
# dimension = "type"
# case = "Untokenizer__compat__token_as_tuple_wrong"
# subject = "lib2to3.pgen2.tokenize.Untokenizer.compat(token: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.tokenize.Untokenizer.compat(token: tuple); call it with the wrong type.

typeshed contract: token is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.compat(12345, None)  # token: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

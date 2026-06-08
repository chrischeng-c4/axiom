# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__compat__token_as_Sequence_wrong"
# subject = "tokenize.Untokenizer.compat(token: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed token"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed token
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.compat(token: Sequence); call it with the wrong type.

typeshed contract: token is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.compat(_W(), None)  # token: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

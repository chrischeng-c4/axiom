# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__untokenize__iterable_as_Iterable_wrong"
# subject = "tokenize.Untokenizer.untokenize(iterable: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.untokenize(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.untokenize(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

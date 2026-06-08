# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__add_backslash_continuation__start_as__Position_wrong"
# subject = "tokenize.Untokenizer.add_backslash_continuation(start: _Position)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.add_backslash_continuation(start: _Position); call it with the wrong type.

typeshed contract: start is _Position. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.add_backslash_continuation(_W())  # start: _Position <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_tokenize"
# dimension = "type"
# case = "Untokenizer__add_whitespace__start_as__Coord_wrong"
# subject = "lib2to3.pgen2.tokenize.Untokenizer.add_whitespace(start: _Coord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.tokenize.Untokenizer.add_whitespace(start: _Coord); call it with the wrong type.

typeshed contract: start is _Coord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.add_whitespace(_W())  # start: _Coord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

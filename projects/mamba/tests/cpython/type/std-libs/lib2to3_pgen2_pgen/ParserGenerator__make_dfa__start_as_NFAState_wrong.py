# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__make_dfa__start_as_NFAState_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.make_dfa(start: NFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.make_dfa(start: NFAState); call it with the wrong type.

typeshed contract: start is NFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.make_dfa(_W(), None)  # start: NFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

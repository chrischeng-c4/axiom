# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_tokenize"
# dimension = "type"
# case = "tokenize__readline_as_Callable_wrong"
# subject = "lib2to3.pgen2.tokenize.tokenize(readline: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed readline"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/tokenize.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed readline
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.tokenize.tokenize(readline: Callable); call it with the wrong type.

typeshed contract: readline is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.tokenize import tokenize
try:
    tokenize(_W())  # readline: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

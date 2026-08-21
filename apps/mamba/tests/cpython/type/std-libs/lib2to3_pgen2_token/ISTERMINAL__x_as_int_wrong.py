# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_token"
# dimension = "type"
# case = "ISTERMINAL__x_as_int_wrong"
# subject = "lib2to3.pgen2.token.ISTERMINAL(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/token.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.token.ISTERMINAL(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.token import ISTERMINAL
try:
    ISTERMINAL("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

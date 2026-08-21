# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "capwords__s_as_StrOrLiteralStr_wrong"
# subject = "string.capwords(s: StrOrLiteralStr)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s
# mamba-strict-type: TypeError
"""Type wall: string.capwords(s: StrOrLiteralStr); call it with the wrong type.

typeshed contract: s is StrOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import capwords
try:
    capwords(_W())  # s: StrOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

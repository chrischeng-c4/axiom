# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "lasti2lineno__code_as_CodeType_wrong"
# subject = "pdb.lasti2lineno(code: CodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pdb.lasti2lineno(code: CodeType); call it with the wrong type.

typeshed contract: code is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pdb import lasti2lineno
try:
    lasti2lineno(_W(), 0)  # code: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

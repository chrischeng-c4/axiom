# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "DFAState__addarc__next_as_DFAState_wrong"
# subject = "lib2to3.pgen2.pgen.DFAState.addarc(next: DFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.DFAState.addarc(next: DFAState); call it with the wrong type.

typeshed contract: next is DFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import DFAState
obj = object.__new__(DFAState)
try:
    obj.addarc(_W(), "")  # next: DFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

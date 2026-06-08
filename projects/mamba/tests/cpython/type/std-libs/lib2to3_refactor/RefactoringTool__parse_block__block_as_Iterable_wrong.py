# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_refactor"
# dimension = "type"
# case = "RefactoringTool__parse_block__block_as_Iterable_wrong"
# subject = "lib2to3.refactor.RefactoringTool.parse_block(block: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/refactor.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.refactor.RefactoringTool.parse_block(block: Iterable); call it with the wrong type.

typeshed contract: block is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.refactor import RefactoringTool
obj = object.__new__(RefactoringTool)
try:
    obj.parse_block(_W(), 0, 0)  # block: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

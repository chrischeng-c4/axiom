# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_refactor"
# dimension = "type"
# case = "RefactoringTool__refactor_doctest__block_as_list_wrong"
# subject = "lib2to3.refactor.RefactoringTool.refactor_doctest(block: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/refactor.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.refactor.RefactoringTool.refactor_doctest(block: list); call it with the wrong type.

typeshed contract: block is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.refactor import RefactoringTool
obj = object.__new__(RefactoringTool)
try:
    obj.refactor_doctest(12345, 0, 0, None)  # block: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__init__fixers_as_Iterable_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.__init__(fixers: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.__init__(fixers: Iterable); call it with the wrong type.

typeshed contract: fixers is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.main import StdoutRefactoringTool
try:
    StdoutRefactoringTool(_W(), None, None, True, True)  # fixers: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

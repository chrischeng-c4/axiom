# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__print_output__old_as_str_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.print_output(old: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.print_output(old: str); call it with the wrong type.

typeshed contract: old is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import StdoutRefactoringTool
obj = object.__new__(StdoutRefactoringTool)
try:
    obj.print_output(12345, "", "", True)  # old: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

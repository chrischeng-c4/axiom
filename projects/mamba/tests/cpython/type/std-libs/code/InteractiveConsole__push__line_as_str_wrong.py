# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__push__line_as_str_wrong"
# subject = "code.InteractiveConsole.push(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.push(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveConsole
obj = object.__new__(InteractiveConsole)
try:
    obj.push(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

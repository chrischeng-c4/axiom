# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__raw_input__prompt_as_str_wrong"
# subject = "code.InteractiveConsole.raw_input(prompt: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.raw_input(prompt: str); call it with the wrong type.

typeshed contract: prompt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveConsole
obj = object.__new__(InteractiveConsole)
try:
    obj.raw_input(12345)  # prompt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "type"
# case = "CommandCompiler____call____source_as_str_wrong"
# subject = "codeop.CommandCompiler.__call__(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codeop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codeop.CommandCompiler.__call__(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codeop import CommandCompiler
obj = object.__new__(CommandCompiler)
try:
    obj.__call__(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "type"
# case = "compile_command__source_as_str_wrong"
# subject = "codeop.compile_command(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codeop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codeop.compile_command(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codeop import compile_command
try:
    compile_command(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

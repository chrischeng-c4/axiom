# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "Instruction__make__opname_as_str_wrong"
# subject = "dis.Instruction.make(opname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.Instruction.make(opname: str); call it with the wrong type.

typeshed contract: opname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dis import Instruction
try:
    Instruction.make(12345, None, None, "", 0, 0, True, None)  # opname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

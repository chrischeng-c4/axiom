# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcode"
# dimension = "type"
# case = "stack_effect__opcode_as_int_wrong"
# subject = "opcode.stack_effect(opcode: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/opcode.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: opcode.stack_effect(opcode: int); call it with the wrong type.

typeshed contract: opcode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from opcode import stack_effect
try:
    stack_effect("not_an_int")  # opcode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

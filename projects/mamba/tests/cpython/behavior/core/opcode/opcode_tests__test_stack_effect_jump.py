# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "opcode"
# dimension = "behavior"
# case = "opcode_tests__test_stack_effect_jump"
# subject = "cpython.test__opcode.OpcodeTests.test_stack_effect_jump"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__opcode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test__opcode.py::OpcodeTests::test_stack_effect_jump
"""Auto-ported test: OpcodeTests::test_stack_effect_jump (CPython 3.12 oracle)."""


import dis
from test.support.import_helper import import_module
import unittest
import opcode
from _opcode import stack_effect


_opcode = import_module('_opcode')


# --- test body ---
FOR_ITER = dis.opmap['FOR_ITER']

assert stack_effect(FOR_ITER, 0) == 1

assert stack_effect(FOR_ITER, 0, jump=True) == 1

assert stack_effect(FOR_ITER, 0, jump=False) == 1
JUMP_FORWARD = dis.opmap['JUMP_FORWARD']

assert stack_effect(JUMP_FORWARD, 0) == 0

assert stack_effect(JUMP_FORWARD, 0, jump=True) == 0

assert stack_effect(JUMP_FORWARD, 0, jump=False) == 0
has_arg = dis.hasarg
has_exc = dis.hasexc
has_jump = dis.hasjabs + dis.hasjrel
for name, code in filter(lambda item: item[0] not in dis.deoptmap, dis.opmap.items()):
    if code >= opcode.MIN_INSTRUMENTED_OPCODE:
        continue
    if code not in has_arg:
        common = stack_effect(code)
        jump = stack_effect(code, jump=True)
        nojump = stack_effect(code, jump=False)
    else:
        common = stack_effect(code, 0)
        jump = stack_effect(code, 0, jump=True)
        nojump = stack_effect(code, 0, jump=False)
    if code in has_jump or code in has_exc:

        assert common == max(jump, nojump)
    else:

        assert jump == common

        assert nojump == common
print("OpcodeTests::test_stack_effect_jump: ok")

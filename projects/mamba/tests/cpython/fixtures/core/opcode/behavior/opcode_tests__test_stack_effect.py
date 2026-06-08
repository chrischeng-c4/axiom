# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "opcode"
# dimension = "behavior"
# case = "opcode_tests__test_stack_effect"
# subject = "cpython.test__opcode.OpcodeTests.test_stack_effect"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__opcode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test__opcode.py::OpcodeTests::test_stack_effect
"""Auto-ported test: OpcodeTests::test_stack_effect (CPython 3.12 oracle)."""


import dis
from test.support.import_helper import import_module
import unittest
import opcode
from _opcode import stack_effect


_opcode = import_module('_opcode')


# --- test body ---

assert stack_effect(dis.opmap['POP_TOP']) == -1

assert stack_effect(dis.opmap['BUILD_SLICE'], 0) == -1

assert stack_effect(dis.opmap['BUILD_SLICE'], 1) == -1

assert stack_effect(dis.opmap['BUILD_SLICE'], 3) == -2

try:
    stack_effect(30000)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    stack_effect(dis.opmap['BUILD_SLICE'])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    stack_effect(dis.opmap['POP_TOP'], 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
has_arg = dis.hasarg
for name, code in filter(lambda item: item[0] not in dis.deoptmap, dis.opmap.items()):
    if code >= opcode.MIN_INSTRUMENTED_OPCODE:
        continue
    if code not in has_arg:
        stack_effect(code)

        try:
            stack_effect(code, 0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
    else:
        stack_effect(code, 0)

        try:
            stack_effect(code)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
for code in set(range(256)) - set(dis.opmap.values()):

    try:
        stack_effect(code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        stack_effect(code, 0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("OpcodeTests::test_stack_effect: ok")

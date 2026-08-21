# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "peepholer"
# dimension = "behavior"
# case = "test_tranforms__test_assignment_idiom_in_comprehensions"
# subject = "cpython.test_peepholer.TestTranforms.test_assignment_idiom_in_comprehensions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_peepholer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_peepholer.py::TestTranforms::test_assignment_idiom_in_comprehensions
"""Auto-ported test: TestTranforms::test_assignment_idiom_in_comprehensions (CPython 3.12 oracle)."""


import dis
from itertools import combinations, product
import sys
import textwrap
import unittest
from test import support
from test.support.bytecode_helper import BytecodeTestCase, CfgOptimizationTestCase


def compile_pattern_with_fast_locals(pattern):
    source = textwrap.dedent(f'\n        def f(x):\n            match x:\n                case {pattern}:\n                    pass\n        ')
    namespace = {}
    exec(source, namespace)
    return namespace['f'].__code__

def count_instr_recursively(f, opname):
    count = 0
    for instr in dis.get_instructions(f):
        if instr.opname == opname:
            count += 1
    if hasattr(f, '__code__'):
        f = f.__code__
    for c in f.co_consts:
        if hasattr(c, 'co_code'):
            count += count_instr_recursively(c, opname)
    return count


# --- test body ---
def check_jump_targets(code):
    instructions = list(dis.get_instructions(code))
    targets = {instr.offset: instr for instr in instructions}
    for instr in instructions:
        if 'JUMP_' not in instr.opname:
            continue
        tgt = targets[instr.argval]
        if tgt.opname in ('JUMP_BACKWARD', 'JUMP_FORWARD'):

            raise AssertionError(f'{instr.opname} at {instr.offset} jumps to {tgt.opname} at {tgt.offset}')
        if instr.opname in ('JUMP_BACKWARD', 'JUMP_FORWARD') and tgt.opname == 'RETURN_VALUE':

            raise AssertionError(f'{instr.opname} at {instr.offset} jumps to {tgt.opname} at {tgt.offset}')

def check_lnotab(code):
    """Check that the lnotab byte offsets are sensible."""
    code = dis._get_code_object(code)
    lnotab = list(dis.findlinestarts(code))
    min_bytecode = min((t[0] for t in lnotab))
    max_bytecode = max((t[0] for t in lnotab))

    assert min_bytecode >= 0

    assert max_bytecode < len(code.co_code)

def listcomp():
    return [y for x in a for y in [f(x)]]

assert count_instr_recursively(listcomp, 'FOR_ITER') == 1

def setcomp():
    return {y for x in a for y in [f(x)]}

assert count_instr_recursively(setcomp, 'FOR_ITER') == 1

def dictcomp():
    return {y: y for x in a for y in [f(x)]}

assert count_instr_recursively(dictcomp, 'FOR_ITER') == 1

def genexpr():
    return (y for x in a for y in [f(x)])

assert count_instr_recursively(genexpr, 'FOR_ITER') == 1
print("TestTranforms::test_assignment_idiom_in_comprehensions: ok")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "peepholer"
# dimension = "behavior"
# case = "test_buglets__test_bpo_45773_pop_jump_if_true"
# subject = "cpython.test_peepholer.TestBuglets.test_bpo_45773_pop_jump_if_true"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_peepholer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_peepholer.py::TestBuglets::test_bpo_45773_pop_jump_if_true
"""Auto-ported test: TestBuglets::test_bpo_45773_pop_jump_if_true (CPython 3.12 oracle)."""


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
compile('while True or spam: pass', '<test>', 'exec')
print("TestBuglets::test_bpo_45773_pop_jump_if_true: ok")

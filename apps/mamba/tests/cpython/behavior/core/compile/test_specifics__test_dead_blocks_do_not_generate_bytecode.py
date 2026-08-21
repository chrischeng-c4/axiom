# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_dead_blocks_do_not_generate_bytecode"
# subject = "cpython.test_compile.TestSpecifics.test_dead_blocks_do_not_generate_bytecode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_dead_blocks_do_not_generate_bytecode
"""Auto-ported test: TestSpecifics::test_dead_blocks_do_not_generate_bytecode (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def unused_block_if():
    if 0:
        return 42

def unused_block_while():
    while 0:
        return 42

def unused_block_if_else():
    if 1:
        return None
    else:
        return 42

def unused_block_while_else():
    while 1:
        return None
    else:
        return 42
funcs = [unused_block_if, unused_block_while, unused_block_if_else, unused_block_while_else]
for func in funcs:
    opcodes = list(dis.get_instructions(func))

    assert len(opcodes) <= 3

    assert 'RETURN_CONST' == opcodes[-1].opname

    assert None == opcodes[-1].argval
print("TestSpecifics::test_dead_blocks_do_not_generate_bytecode: ok")

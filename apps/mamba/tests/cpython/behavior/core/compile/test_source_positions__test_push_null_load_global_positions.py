# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_source_positions__test_push_null_load_global_positions"
# subject = "cpython.test_compile.TestSourcePositions.test_push_null_load_global_positions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSourcePositions::test_push_null_load_global_positions
"""Auto-ported test: TestSourcePositions::test_push_null_load_global_positions (CPython 3.12 oracle)."""


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
def assertOpcodeSourcePositionIs(code, opcode, line, end_line, column, end_column, occurrence=1):
    for instr, position in zip(dis.Bytecode(code, show_caches=True), code.co_positions(), strict=True):
        if instr.opname == opcode:
            occurrence -= 1
            if not occurrence:

                assert position[0] == line

                assert position[1] == end_line

                assert position[2] == column

                assert position[3] == end_column
                return

    raise AssertionError(f'Opcode {opcode} not found in code')

def check_positions_against_ast(snippet):
    code = compile(snippet, 'test_compile.py', 'exec')
    ast_tree = compile(snippet, 'test_compile.py', 'exec', _ast.PyCF_ONLY_AST)

    assert type(ast_tree) == _ast.Module
    lines, end_lines, columns, end_columns = (set(), set(), set(), set())

    class SourceOffsetVisitor(ast.NodeVisitor):

        def generic_visit(self, node):
            super().generic_visit(node)
            if not isinstance(node, (ast.expr, ast.stmt, ast.pattern)):
                return
            lines.add(node.lineno)
            end_lines.add(node.end_lineno)
            columns.add(node.col_offset)
            end_columns.add(node.end_col_offset)
    SourceOffsetVisitor().visit(ast_tree)
    for line, end_line, col, end_col in code.co_positions():
        if line == 0:
            continue
        if line is not None:

            assert line in lines
        if end_line is not None:

            assert end_line in end_lines
        if col is not None:

            assert col in columns
        if end_col is not None:

            assert end_col in end_columns
    return (code, ast_tree)
source_template = '\n        import abc, dis\n        import ast as art\n\n        abc = None\n        dix = dis\n        ast = art\n\n        def f():\n        {}\n        '
for body in ['    abc.a()', '    art.a()', '    ast.a()', '    dis.a()', '    dix.a()', '    abc[...]()', '    art()()', '   (ast or ...)()', '   [dis]()', '   (dix + ...)()']:
    namespace = {}
    source = textwrap.dedent(source_template.format(body))
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', SyntaxWarning)
        exec(source, namespace)
    code = namespace['f'].__code__
    assertOpcodeSourcePositionIs(code, 'LOAD_GLOBAL', line=10, end_line=10, column=4, end_column=7)
print("TestSourcePositions::test_push_null_load_global_positions: ok")

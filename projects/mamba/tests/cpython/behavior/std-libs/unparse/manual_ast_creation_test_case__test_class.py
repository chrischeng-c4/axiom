# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unparse"
# dimension = "behavior"
# case = "manual_ast_creation_test_case__test_class"
# subject = "cpython.test_unparse.ManualASTCreationTestCase.test_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unparse.py::ManualASTCreationTestCase::test_class
"""Auto-ported test: ManualASTCreationTestCase::test_class (CPython 3.12 oracle)."""


import unittest
import test.support
import pathlib
import random
import tokenize
import ast
from test.support.ast_helper import ASTTestMixin


'Tests for ast.unparse.'

def read_pyfile(filename):
    """Read and return the contents of a Python source file (as a
    string), taking into account the file encoding."""
    with tokenize.open(filename) as stream:
        return stream.read()

for_else = 'def f():\n    for x in range(10):\n        break\n    else:\n        y = 2\n    z = 3\n'

while_else = 'def g():\n    while True:\n        break\n    else:\n        y = 2\n    z = 3\n'

relative_import = 'from . import fred\nfrom .. import barney\nfrom .australia import shrimp as prawns\n'

nonlocal_ex = 'def f():\n    x = 1\n    def g():\n        nonlocal x\n        x = 2\n        y = 7\n        def h():\n            nonlocal x, y\n'

raise_from = 'try:\n    1 / 0\nexcept ZeroDivisionError as e:\n    raise ArithmeticError from e\n'

class_decorator = '@f1(arg)\n@f2\nclass Foo: pass\n'

elif1 = 'if cond1:\n    suite1\nelif cond2:\n    suite2\nelse:\n    suite3\n'

elif2 = 'if cond1:\n    suite1\nelif cond2:\n    suite2\n'

try_except_finally = 'try:\n    suite1\nexcept ex1:\n    suite2\nexcept ex2:\n    suite3\nelse:\n    suite4\nfinally:\n    suite5\n'

try_except_star_finally = 'try:\n    suite1\nexcept* ex1:\n    suite2\nexcept* ex2:\n    suite3\nelse:\n    suite4\nfinally:\n    suite5\n'

with_simple = 'with f():\n    suite1\n'

with_as = 'with f() as x:\n    suite1\n'

with_two_items = 'with f() as x, g() as y:\n    suite1\n'

docstring_prefixes = ('', 'class foo:\n    ', 'def foo():\n    ', 'async def foo():\n    ')

class ASTTestCase(ASTTestMixin, unittest.TestCase):

    def check_ast_roundtrip(self, code1, **kwargs):
        with self.subTest(code1=code1, ast_parse_kwargs=kwargs):
            ast1 = ast.parse(code1, **kwargs)
            code2 = ast.unparse(ast1)
            ast2 = ast.parse(code2, **kwargs)
            self.assertASTEqual(ast1, ast2)

    def check_invalid(self, node, raises=ValueError):
        with self.subTest(node=node):
            self.assertRaises(raises, ast.unparse, node)

    def get_source(self, code1, code2=None):
        code2 = code2 or code1
        code1 = ast.unparse(ast.parse(code1))
        return (code1, code2)

    def check_src_roundtrip(self, code1, code2=None):
        code1, code2 = self.get_source(code1, code2)
        with self.subTest(code1=code1, code2=code2):
            self.assertEqual(code2, code1)

    def check_src_dont_roundtrip(self, code1, code2=None):
        code1, code2 = self.get_source(code1, code2)
        with self.subTest(code1=code1, code2=code2):
            self.assertNotEqual(code2, code1)


# --- test body ---
node = ast.ClassDef(name='X', bases=[], keywords=[], body=[ast.Pass()], decorator_list=[])
ast.fix_missing_locations(node)

assert ast.unparse(node) == 'class X:\n    pass'
print("ManualASTCreationTestCase::test_class: ok")

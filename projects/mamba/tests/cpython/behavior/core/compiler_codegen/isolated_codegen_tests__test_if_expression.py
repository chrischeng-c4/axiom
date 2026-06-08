# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compiler_codegen"
# dimension = "behavior"
# case = "isolated_codegen_tests__test_if_expression"
# subject = "cpython.test_compiler_codegen.IsolatedCodeGenTests.test_if_expression"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compiler_codegen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""IsolatedCodeGenTests.test_if_expression: AST codegen emits branch layout."""
import ast
from test.support.bytecode_helper import CodegenTestCase

case = CodegenTestCase()
snippet = "42 if True else 24"
tree = ast.parse(snippet, "my_file.py", "exec")
instructions = case.generate_code(tree)

false_label = case.Label()
exit_label = case.Label()
expected = [
    ("RESUME", 0, 0),
    ("LOAD_CONST", 0, 1),
    ("POP_JUMP_IF_FALSE", false_label, 1),
    ("LOAD_CONST", 1, 1),
    ("JUMP", exit_label),
    false_label,
    ("LOAD_CONST", 2, 1),
    exit_label,
    ("POP_TOP", None),
    ("LOAD_CONST", 3),
    ("RETURN_VALUE", None),
]

case.assertInstructionsMatch(instructions, expected)

print("IsolatedCodeGenTests::test_if_expression: ok")

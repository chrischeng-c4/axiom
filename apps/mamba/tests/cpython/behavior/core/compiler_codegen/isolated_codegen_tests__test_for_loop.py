# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compiler_codegen"
# dimension = "behavior"
# case = "isolated_codegen_tests__test_for_loop"
# subject = "cpython.test_compiler_codegen.IsolatedCodeGenTests.test_for_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compiler_codegen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""IsolatedCodeGenTests.test_for_loop: AST codegen emits loop bytecode layout."""
import ast
from test.support.bytecode_helper import CodegenTestCase

case = CodegenTestCase()
snippet = "for x in l:\n\tprint(x)"
tree = ast.parse(snippet, "my_file.py", "exec")
instructions = case.generate_code(tree)

loop_label = case.Label()
exit_label = case.Label()
expected = [
    ("RESUME", 0, 0),
    ("LOAD_NAME", 0, 1),
    ("GET_ITER", None, 1),
    loop_label,
    ("FOR_ITER", exit_label, 1),
    ("NOP", None, 1, 1),
    ("STORE_NAME", 1, 1),
    ("PUSH_NULL", None, 2),
    ("LOAD_NAME", 2, 2),
    ("LOAD_NAME", 1, 2),
    ("CALL", 1, 2),
    ("POP_TOP", None),
    ("JUMP", loop_label),
    exit_label,
    ("END_FOR", None),
    ("LOAD_CONST", 0),
    ("RETURN_VALUE", None),
]

case.assertInstructionsMatch(instructions, expected)

print("IsolatedCodeGenTests::test_for_loop: ok")

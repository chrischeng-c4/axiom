# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compiler_assemble"
# dimension = "behavior"
# case = "isolated_assemble_tests__test_simple_expr"
# subject = "cpython.test_compiler_assemble.IsolatedAssembleTests.test_simple_expr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compiler_assemble.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""IsolatedAssembleTests.test_simple_expr: assemble code object for avg."""
import types
from test.support.bytecode_helper import AssemblerTestCase

case = AssemblerTestCase()

metadata = {
    "filename": "avg.py",
    "name": "avg",
    "qualname": "stats.avg",
    "consts": {2: 0},
    "argcount": 2,
    "varnames": {"x": 0, "y": 1},
}
for key in ["cellvars", "freevars", "fasthidden", "names"]:
    metadata.setdefault(key, {})
for key in ["posonlyargcount", "kwonlyargcount"]:
    metadata.setdefault(key, 0)
metadata.setdefault("firstlineno", 1)

instructions = [
    ("RESUME", 0),
    ("LOAD_FAST", 0, 1),
    ("LOAD_FAST", 1, 1),
    ("BINARY_OP", 0, 1),
    ("LOAD_CONST", 0, 1),
    ("BINARY_OP", 11, 1),
    ("RETURN_VALUE", 1),
]
complete_instructions = case.complete_insts_info(instructions)
code = case.get_code_object(metadata["filename"], complete_instructions, metadata)

assert isinstance(code, types.CodeType), type(code)
assert code.co_filename == "avg.py", code.co_filename
assert code.co_name == "avg", code.co_name
assert code.co_qualname == "stats.avg", code.co_qualname
assert code.co_consts == (2,), code.co_consts
assert code.co_argcount == 2, code.co_argcount
assert code.co_varnames == ("x", "y"), code.co_varnames

avg = types.FunctionType(code, {})
assert avg(3, 4) == 3.5
assert avg(-100, 200) == 50
assert avg(10, 18) == 14

print("IsolatedAssembleTests::test_simple_expr: ok")

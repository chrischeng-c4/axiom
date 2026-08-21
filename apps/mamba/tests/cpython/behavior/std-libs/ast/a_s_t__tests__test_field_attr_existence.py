# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_field_attr_existence"
# subject = "cpython.test_ast.AST_Tests.test_field_attr_existence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


def is_ast_node(name, node):
    if not isinstance(node, type):
        return False
    if "ast" not in node.__module__:
        return False
    return name != "AST" and name[0].isupper()


for name, item in ast.__dict__.items():
    if name in {"Num", "Str", "Bytes", "NameConstant", "Ellipsis"}:
        continue
    if name == "Index":
        continue
    if is_ast_node(name, item):
        node = item()
        if isinstance(node, ast.AST) and type(node._fields) is not tuple:
            raise AssertionError((name, node._fields))

print("AST_Tests::test_field_attr_existence: ok")

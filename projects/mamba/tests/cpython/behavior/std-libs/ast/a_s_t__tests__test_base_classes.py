# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_base_classes"
# subject = "cpython.test_ast.AST_Tests.test_base_classes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

checks = [
    ("For_stmt", ast.For, ast.stmt),
    ("Name_expr", ast.Name, ast.expr),
    ("stmt_AST", ast.stmt, ast.AST),
    ("expr_AST", ast.expr, ast.AST),
    ("comprehension_AST", ast.comprehension, ast.AST),
    ("Gt_AST", ast.Gt, ast.AST),
]

for label, child, parent in checks:
    result = issubclass(child, parent)
    if not result:
        raise AssertionError(label)
print("AST_Tests::test_base_classes: ok")

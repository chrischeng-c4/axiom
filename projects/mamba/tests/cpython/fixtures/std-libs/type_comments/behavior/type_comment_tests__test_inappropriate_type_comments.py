# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_comments"
# dimension = "behavior"
# case = "type_comment_tests__test_inappropriate_type_comments"
# subject = "cpython.test_type_comments.TypeCommentTests.test_inappropriate_type_comments"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_comments.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_comments.py::TypeCommentTests::test_inappropriate_type_comments
"""Auto-ported test: TypeCommentTests::test_inappropriate_type_comments."""


import ast
import sys


lowest = 4
highest = sys.version_info[1]


def parse(source, feature_version=highest):
    return ast.parse(source, type_comments=True, feature_version=feature_version)


def parse_all(source, minver=lowest, maxver=highest):
    for version in range(lowest, highest + 1):
        feature_version = (3, version)
        if minver <= version <= maxver:
            parse(source, feature_version)
        else:
            try:
                parse(source, feature_version)
            except SyntaxError:
                pass
            else:
                raise AssertionError(f"expected SyntaxError for feature_version={feature_version}")


def check_both_ways(source):
    ast.parse(source, type_comments=False)
    parse_all(source, maxver=0)


check_both_ways("pass  # type: int\n")
check_both_ways("foo()  # type: int\n")
check_both_ways("x += 1  # type: int\n")
check_both_ways("while True:  # type: int\n  continue\n")
check_both_ways("while True:\n  continue  # type: int\n")
check_both_ways("try:  # type: int\n  pass\nfinally:\n  pass\n")
check_both_ways("try:\n  pass\nfinally:  # type: int\n  pass\n")
check_both_ways("pass  # type: ignorewhatever\n")
check_both_ways("pass  # type: ignore\u00e9\n")

print("TypeCommentTests::test_inappropriate_type_comments: ok")

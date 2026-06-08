# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_18"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_18"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_18
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_18 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
class TwoDimensionalList:

    def __init__(self, two_dimensional_list):
        self.two_dimensional_list = two_dimensional_list

    def __getitem__(self, index):
        return self.two_dimensional_list[index[0]][index[1]]
a = TwoDimensionalList([[1], [2]])
element = a[(b := 0), (c := 0)]

assert b == 0

assert c == 0

assert element == a.two_dimensional_list[b][c]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_18: ok")

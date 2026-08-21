# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_filter_binding_reused_and_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a comprehension filter binds per-element, is reusable in that element's value expression, and the last filter binding leaks to the enclosing scope"""
# A walrus in the comprehension's filter binds per-element and is reusable
# in the value expression of the same element.
def positive(value):
    return value

rows = [(x, y, x / y) for x in [1, 2, 3] if (y := positive(x)) > 0]
assert rows == [(1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)]
# The last filter binding leaks to the enclosing scope.
assert y == 3

print("comp_filter_binding_reused_and_leaks OK")

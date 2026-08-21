# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_value_binding_reused_in_element"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus bound in a comprehension's value expression can be reused later in the same element, and the final binding leaks to the enclosing scope"""
# The walrus value can be reused later in the element expression.
def positive(value):
    return value

ratios = [[(z := positive(n)), n / z] for n in range(1, 5)]
assert ratios == [[1, 1.0], [2, 1.0], [3, 1.0], [4, 1.0]]
# The final binding leaks to the enclosing scope.
assert z == 4

print("comp_value_binding_reused_in_element OK")

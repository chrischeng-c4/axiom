# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "class_body_comprehension_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: class_body_comprehension_rejected (errors)."""
pass

_raised = False
try:
    compile('class Foo:\n    [(42, j := i) for i in range(5)]\n', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "class_body_comprehension_rejected: expected SyntaxError"
print("class_body_comprehension_rejected OK")

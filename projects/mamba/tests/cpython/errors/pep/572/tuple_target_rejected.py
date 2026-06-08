# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "tuple_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: tuple_target_rejected (errors)."""
pass

_raised = False
try:
    compile('((a, b) := (1, 2))', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "tuple_target_rejected: expected SyntaxError"
print("tuple_target_rejected OK")

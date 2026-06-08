# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "rebind_iteration_variable_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: rebind_iteration_variable_rejected (errors)."""
pass

_raised = False
try:
    compile('[[(__x := 2) for _ in range(2)] for __x in range(2)]', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "rebind_iteration_variable_rejected: expected SyntaxError"
print("rebind_iteration_variable_rejected OK")

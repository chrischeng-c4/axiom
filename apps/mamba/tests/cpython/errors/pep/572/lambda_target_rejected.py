# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "lambda_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: lambda_target_rejected (errors)."""
pass

_raised = False
try:
    compile('(lambda: x := 1)', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "lambda_target_rejected: expected SyntaxError"
print("lambda_target_rejected OK")

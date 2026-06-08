# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "subscript_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: subscript_target_rejected (errors)."""
pass

_raised = False
try:
    compile("(d := {}); (d['k'] := 1)", '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "subscript_target_rejected: expected SyntaxError"
print("subscript_target_rejected OK")

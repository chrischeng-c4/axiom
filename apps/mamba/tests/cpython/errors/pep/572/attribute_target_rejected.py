# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "attribute_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: attribute_target_rejected (errors)."""
pass

_raised = False
try:
    compile('(o.x := 5)', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "attribute_target_rejected: expected SyntaxError"
print("attribute_target_rejected OK")

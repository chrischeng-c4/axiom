# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "comprehension_iterable_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: comprehension_iterable_target_rejected (errors)."""
pass

_raised = False
try:
    compile('[i + 1 for i in i := [1, 2]]', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "comprehension_iterable_target_rejected: expected SyntaxError"
print("comprehension_iterable_target_rejected OK")

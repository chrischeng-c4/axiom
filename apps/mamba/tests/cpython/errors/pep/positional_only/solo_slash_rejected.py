# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "solo_slash_rejected"
# subject = "/"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: solo_slash_rejected (errors)."""
pass

_raised = False
try:
    compile('def h(/): pass', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "solo_slash_rejected: expected SyntaxError"
print("solo_slash_rejected OK")

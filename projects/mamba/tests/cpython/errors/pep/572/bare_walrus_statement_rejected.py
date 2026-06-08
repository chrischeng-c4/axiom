# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "bare_walrus_statement_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: bare_walrus_statement_rejected (errors)."""
pass

_raised = False
try:
    compile('a := 5', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "bare_walrus_statement_rejected: expected SyntaxError"
print("bare_walrus_statement_rejected OK")

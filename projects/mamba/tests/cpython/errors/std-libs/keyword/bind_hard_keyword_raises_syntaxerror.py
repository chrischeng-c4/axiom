# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "errors"
# case = "bind_hard_keyword_raises_syntaxerror"
# subject = "keyword.kwlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.kwlist: bind_hard_keyword_raises_syntaxerror (errors)."""
import keyword

_raised = False
try:
    exec("if = 42")
except SyntaxError:
    _raised = True
assert _raised, "bind_hard_keyword_raises_syntaxerror: expected SyntaxError"
print("bind_hard_keyword_raises_syntaxerror OK")

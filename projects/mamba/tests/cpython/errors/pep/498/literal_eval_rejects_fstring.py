# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "literal_eval_rejects_fstring"
# subject = "ast.literal_eval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ast.literal_eval: literal_eval_rejects_fstring (errors)."""
import ast

_raised = False
try:
    ast.literal_eval("f'x'")
except ValueError:
    _raised = True
assert _raised, "literal_eval_rejects_fstring: expected ValueError"
print("literal_eval_rejects_fstring OK")

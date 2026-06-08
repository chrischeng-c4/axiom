# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "errors"
# case = "bad_annotation_syntax_raises"
# subject = "exec"
# kind = "mechanical"
# xfail = "mamba exec defers parsing and returns None silently instead of raising SyntaxError. See project_mamba_eval_silent_none_cross_type."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""exec: bad_annotation_syntax_raises (errors)."""
import sys

_raised = False
try:
    exec('x: not::a::valid::type = 1')
except SyntaxError:
    _raised = True
assert _raised, "bad_annotation_syntax_raises: expected SyntaxError"
print("bad_annotation_syntax_raises OK")

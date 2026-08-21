# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "kwonly_as_positional_rejected"
# subject = "*"
# kind = "mechanical"
# xfail = "mamba does not enforce keyword-only parameters: `_k(3, 4)` is accepted silently instead of raising TypeError (project_mamba_function_machinery_silent_divergences #3)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""*: kwonly_as_positional_rejected (errors)."""
exec("def _k(*, n, m):\n    return n * m", globals())

_raised = False
try:
    _k(3, 4)
except TypeError:
    _raised = True
assert _raised, "kwonly_as_positional_rejected: expected TypeError"
print("kwonly_as_positional_rejected OK")

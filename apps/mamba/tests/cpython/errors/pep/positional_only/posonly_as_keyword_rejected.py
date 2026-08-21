# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "posonly_as_keyword_rejected"
# subject = "/"
# kind = "mechanical"
# xfail = "mamba does not enforce positional-only parameters: `_p(a=1, b=2)` is accepted silently instead of raising TypeError (project_mamba_function_machinery_silent_divergences #4)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: posonly_as_keyword_rejected (errors)."""
exec("def _p(a, b, /):\n    return a + b", globals())

_raised = False
try:
    _p(a=1, b=2)
except TypeError:
    _raised = True
assert _raised, "posonly_as_keyword_rejected: expected TypeError"
print("posonly_as_keyword_rejected OK")

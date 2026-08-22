# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "fstring_rejects_object"
# subject = "fstring.float_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: fstring_rejects_object (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    f'{object():.3f}'
except TypeError:
    _raised = True
assert _raised, "fstring_rejects_object: expected TypeError"
print("fstring_rejects_object OK")

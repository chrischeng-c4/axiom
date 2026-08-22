# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "fstring_dunder_float_without_dunder_format_rejected"
# subject = "fstring.float_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: fstring_dunder_float_without_dunder_format_rejected (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    f'{type("FloatOnly", (), {"__float__": lambda self: 2.5})():.3f}'
except TypeError:
    _raised = True
assert _raised, "fstring_dunder_float_without_dunder_format_rejected: expected TypeError"
print("fstring_dunder_float_without_dunder_format_rejected OK")

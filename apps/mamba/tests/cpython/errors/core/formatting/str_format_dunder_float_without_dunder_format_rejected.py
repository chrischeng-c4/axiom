# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_dunder_float_without_dunder_format_rejected"
# subject = "str.format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str_format_dunder_float_without_dunder_format_rejected (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{:.3f}'.format(type('FloatOnly', (), {'__float__': lambda self: 2.5})())
except TypeError:
    _raised = True
assert _raised, "str_format_dunder_float_without_dunder_format_rejected: expected TypeError"
print("str_format_dunder_float_without_dunder_format_rejected OK")

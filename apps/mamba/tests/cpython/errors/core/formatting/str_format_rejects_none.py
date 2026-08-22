# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_rejects_none"
# subject = "str.format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str_format_rejects_none (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{:.3f}'.format(None)
except TypeError:
    _raised = True
assert _raised, "str_format_rejects_none: expected TypeError"
print("str_format_rejects_none OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_rejects_object"
# subject = "str.format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str_format_rejects_object (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{:.3f}'.format(object())
except TypeError:
    _raised = True
assert _raised, "str_format_rejects_object: expected TypeError"
print("str_format_rejects_object OK")

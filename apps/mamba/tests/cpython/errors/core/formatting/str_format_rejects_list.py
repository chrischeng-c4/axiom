# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_rejects_list"
# subject = "str.format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str_format_rejects_list (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{:.3f}'.format([1])
except TypeError:
    _raised = True
assert _raised, "str_format_rejects_list: expected TypeError"
print("str_format_rejects_list OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_string_receiver_keeps_value_error"
# subject = "str.format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str_format_string_receiver_keeps_value_error (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{:.3f}'.format('abc')
except ValueError:
    _raised = True
assert _raised, "str_format_string_receiver_keeps_value_error: expected ValueError"
print("str_format_string_receiver_keeps_value_error OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "fstring_string_receiver_keeps_value_error"
# subject = "fstring.float_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: fstring_string_receiver_keeps_value_error (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    f'{"abc":.3f}'
except ValueError:
    _raised = True
assert _raised, "fstring_string_receiver_keeps_value_error: expected ValueError"
print("fstring_string_receiver_keeps_value_error OK")

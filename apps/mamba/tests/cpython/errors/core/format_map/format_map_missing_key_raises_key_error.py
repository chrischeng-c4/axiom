# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "format_map"
# dimension = "errors"
# case = "format_map_missing_key_raises_key_error"
# subject = "str.format_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format_map: format_map_missing_key_raises_key_error (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '{x}'.format_map({})
except KeyError:
    _raised = True
assert _raised, "format_map_missing_key_raises_key_error: expected KeyError"
print("format_map_missing_key_raises_key_error OK")

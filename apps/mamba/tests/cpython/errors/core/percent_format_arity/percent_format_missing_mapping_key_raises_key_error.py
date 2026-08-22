# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "errors"
# case = "percent_format_missing_mapping_key_raises_key_error"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_format_missing_mapping_key_raises_key_error (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%(name)s' % {}
except KeyError:
    _raised = True
assert _raised, "percent_format_missing_mapping_key_raises_key_error: expected KeyError"
print("percent_format_missing_mapping_key_raises_key_error OK")

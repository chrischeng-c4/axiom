# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "errors"
# case = "percent_format_rejects_long_tuple"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_format_rejects_long_tuple (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%s' % (1, 2)
except TypeError:
    _raised = True
assert _raised, "percent_format_rejects_long_tuple: expected TypeError"
print("percent_format_rejects_long_tuple OK")

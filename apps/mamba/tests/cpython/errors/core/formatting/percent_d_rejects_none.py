# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_d_rejects_none"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_d_rejects_none (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%d' % None
except TypeError:
    _raised = True
assert _raised, "percent_d_rejects_none: expected TypeError"
print("percent_d_rejects_none OK")

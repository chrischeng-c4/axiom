# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_upper_e_rejects_object"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_upper_e_rejects_object (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%.3E' % object()
except TypeError:
    _raised = True
assert _raised, "percent_upper_e_rejects_object: expected TypeError"
print("percent_upper_e_rejects_object OK")

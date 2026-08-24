# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_f_rejects_str"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_f_rejects_str (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%.3f' % 'abc'
except TypeError:
    _raised = True
assert _raised, "percent_f_rejects_str: expected TypeError"
print("percent_f_rejects_str OK")

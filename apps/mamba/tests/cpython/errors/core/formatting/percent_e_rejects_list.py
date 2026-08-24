# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_e_rejects_list"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_e_rejects_list (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%.3e' % [1]
except TypeError:
    _raised = True
assert _raised, "percent_e_rejects_list: expected TypeError"
print("percent_e_rejects_list OK")

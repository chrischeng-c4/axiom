# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "errors"
# case = "percent_format_rejects_non_mapping_operand"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_format_rejects_non_mapping_operand (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    '%(name)s' % 'Ada'
except TypeError:
    _raised = True
assert _raised, "percent_format_rejects_non_mapping_operand: expected TypeError"
print("percent_format_rejects_non_mapping_operand OK")

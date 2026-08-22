# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "errors"
# case = "percent_format_rejects_unused_positional_args"
# subject = "str.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent_format_rejects_unused_positional_args (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    'literal' % (1,)
except TypeError:
    _raised = True
assert _raised, "percent_format_rejects_unused_positional_args: expected TypeError"
print("percent_format_rejects_unused_positional_args OK")

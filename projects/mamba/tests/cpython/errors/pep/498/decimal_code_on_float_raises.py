# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "decimal_code_on_float_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':d' on float) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: decimal_code_on_float_raises (errors)."""
# 'd' (decimal-integer) applied to a float is a ValueError

_raised = False
try:
    eval("f'{1.5:d}'")
except ValueError:
    _raised = True
assert _raised, "decimal_code_on_float_raises: expected ValueError"
print("decimal_code_on_float_raises OK")

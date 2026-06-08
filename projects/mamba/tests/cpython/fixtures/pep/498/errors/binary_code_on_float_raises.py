# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "binary_code_on_float_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':b' on float) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: binary_code_on_float_raises (errors)."""
# 'b' (binary) applied to a float is a ValueError

_raised = False
try:
    '{:b}'.format(1.5)
except ValueError:
    _raised = True
assert _raised, "binary_code_on_float_raises: expected ValueError"
print("binary_code_on_float_raises OK")

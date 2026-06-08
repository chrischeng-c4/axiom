# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "string_code_on_int_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':s' on int) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: string_code_on_int_raises (errors)."""
# 's' (string) applied to an int is a ValueError

_raised = False
try:
    eval("f'{1:s}'")
except ValueError:
    _raised = True
assert _raised, "string_code_on_int_raises: expected ValueError"
print("string_code_on_int_raises OK")

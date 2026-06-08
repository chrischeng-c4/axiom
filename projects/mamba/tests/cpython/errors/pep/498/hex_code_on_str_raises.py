# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "hex_code_on_str_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':x' on str) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: hex_code_on_str_raises (errors)."""
# 'x' (hex) applied to a str is a ValueError

_raised = False
try:
    '{:x}'.format('a')
except ValueError:
    _raised = True
assert _raised, "hex_code_on_str_raises: expected ValueError"
print("hex_code_on_str_raises OK")

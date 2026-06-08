# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "comma_underscore_grouping_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba eval() returns None for the mutually-exclusive ',' and '_' grouping options instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: comma_underscore_grouping_raises (errors)."""
# ',' and '_' grouping options are mutually exclusive (ValueError)

_raised = False
try:
    eval("f'{1:,_}'")
except ValueError:
    _raised = True
assert _raised, "comma_underscore_grouping_raises: expected ValueError"
print("comma_underscore_grouping_raises OK")

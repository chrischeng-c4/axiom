# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "bad_format_spec_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba eval() defers parsing past the strict-typing gate; an invalid format code returns None instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27, project_mamba_eval_silent_none_cross_type)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: bad_format_spec_raises (errors)."""
# an invalid presentation type in the format spec raises ValueError

_raised = False
try:
    eval('f"{1:Q}"')
except ValueError:
    _raised = True
assert _raised, "bad_format_spec_raises: expected ValueError"
print("bad_format_spec_raises OK")

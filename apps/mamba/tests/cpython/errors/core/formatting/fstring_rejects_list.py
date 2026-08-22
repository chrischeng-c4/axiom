# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "fstring_rejects_list"
# subject = "fstring.float_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: fstring_rejects_list (errors)."""
# formatting syntax uses only Python built-ins

_raised = False
try:
    f'{[1]:.3f}'
except TypeError:
    _raised = True
assert _raised, "fstring_rejects_list: expected TypeError"
print("fstring_rejects_list OK")

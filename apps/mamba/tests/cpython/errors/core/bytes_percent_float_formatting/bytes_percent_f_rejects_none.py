# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "errors"
# case = "bytes_percent_f_rejects_none"
# subject = "bytes.percent_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes_percent_f_rejects_none (errors)."""
# bytes formatting syntax uses only Python built-ins

_raised = False
try:
    b'%.3f' % None
except TypeError:
    _raised = True
assert _raised, "bytes_percent_f_rejects_none: expected TypeError"
print("bytes_percent_f_rejects_none OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_static_format_no_self_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; unbound .format() does not raise (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: formatter_static_format_no_self_raises (errors)."""
import string

_raised = False
try:
    string.Formatter.format()
except TypeError:
    _raised = True
assert _raised, "formatter_static_format_no_self_raises: expected TypeError"
print("formatter_static_format_no_self_raises OK")

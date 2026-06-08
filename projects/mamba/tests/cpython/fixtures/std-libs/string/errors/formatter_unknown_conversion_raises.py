# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_unknown_conversion_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_unknown_conversion_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().format('{0!h}', 'test')
except ValueError:
    _raised = True
assert _raised, "formatter_unknown_conversion_raises: expected ValueError"
print("formatter_unknown_conversion_raises OK")

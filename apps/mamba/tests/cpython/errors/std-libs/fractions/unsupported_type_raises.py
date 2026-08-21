# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "unsupported_type_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: unsupported_type_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction([1, 2])
except TypeError:
    _raised = True
assert _raised, "unsupported_type_raises: expected TypeError"
print("unsupported_type_raises OK")

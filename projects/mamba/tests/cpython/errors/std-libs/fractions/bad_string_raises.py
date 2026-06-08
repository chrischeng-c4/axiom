# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "bad_string_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: bad_string_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction('not_a_number')
except ValueError:
    _raised = True
assert _raised, "bad_string_raises: expected ValueError"
print("bad_string_raises OK")

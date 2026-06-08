# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "longexp"
# dimension = "behavior"
# case = "long_exp_text__test_longexp"
# subject = "cpython.test_longexp.LongExpText.test_longexp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_longexp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_longexp.py::LongExpText::test_longexp
"""Auto-ported test: LongExpText::test_longexp (CPython 3.12 oracle)."""


import unittest


# --- test body ---
REPS = 65580
l = eval('[' + '2,' * REPS + ']')

assert len(l) == REPS
print("LongExpText::test_longexp: ok")

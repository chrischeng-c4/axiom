# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "localcontext_restores_and_copies"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext yields a fresh copy that is the active context inside the block, then restores the previous context exactly on exit"""
from decimal import getcontext, localcontext

# localcontext yields a fresh copy and restores the previous context on exit.
orig = getcontext()
with localcontext() as entered:
    inside = getcontext()
    assert inside is entered, "getcontext() inside is the entered context"
    assert orig is not entered, "entered context is a copy"
assert getcontext() is orig, "context restored on exit"

print("localcontext_restores_and_copies OK")

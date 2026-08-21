# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "naive_aware_compare_raises"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: comparing a naive datetime against an aware one with `<` raises TypeError (CPython refuses to order across the naive/aware boundary)"""
import datetime

naive = datetime.datetime(2024, 1, 1)
aware = datetime.datetime(2024, 1, 1, tzinfo=datetime.timezone.utc)
_raised = False
try:
    _ = naive < aware
except TypeError:
    _raised = True
assert _raised, "naive_aware_compare_raises: expected TypeError"
print("naive_aware_compare_raises OK")

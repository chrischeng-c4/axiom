# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_unary_plus_minus_filter_signs"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: unary + keeps only positive counts; unary - flips signs and keeps the now-positive ones"""
from collections import Counter

c = Counter(a=-5, b=0, c=5, d=10, e=15, g=40)
assert dict(+c) == dict(c=5, d=10, e=15, g=40), f"+c keeps positives = {dict(+c)!r}"
assert dict(-c) == dict(a=5), f"-c flips signs and keeps positives = {dict(-c)!r}"

print("counter_unary_plus_minus_filter_signs OK")

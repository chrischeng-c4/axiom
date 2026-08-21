# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_subtract_keeps_negative_results"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.subtract keeps negative results (unlike __sub__): subtracting overlapping counts and a string of elements yields the expected signed tallies"""
from collections import Counter

sub = Counter(a=-5, b=0, c=5, d=10, e=15, g=40)
sub.subtract(a=1, b=2, c=-3, d=10, e=20, f=30, h=-50)
assert sub == Counter(a=-6, b=-2, c=8, d=0, e=-5, f=-30, g=40, h=50), f"subtract = {dict(sub)!r}"
sc = Counter("aaabbcd")
sc.subtract("aaaabbcce")
assert sc == Counter(a=-1, b=0, c=-1, d=1, e=-1), f"subtract from string = {dict(sc)!r}"

print("counter_subtract_keeps_negative_results OK")

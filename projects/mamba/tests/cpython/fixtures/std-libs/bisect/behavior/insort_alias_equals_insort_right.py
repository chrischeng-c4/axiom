# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_alias_equals_insort_right"
# subject = "bisect.insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: insort is an alias for insort_right and produces the identical list"""
import bisect

via_insort = [1, 2, 3]
bisect.insort(via_insort, 2)
via_right = [1, 2, 3]
bisect.insort_right(via_right, 2)
assert via_insort == via_right, f"insort == insort_right ({via_insort!r} vs {via_right!r})"

print("insort_alias_equals_insort_right OK")

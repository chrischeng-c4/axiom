# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mode_single_most_common"
# subject = "statistics.mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mode: mode returns the single most common value over any hashable iterable (ints, characters, words) and on a tie returns the first-seen value, never erroring"""
import collections
from statistics import mode

# The single most common value, over any hashable iterable.
assert mode([1, 2, 2, 3]) == 2, mode([1, 2, 2, 3])
assert mode("abcbdb") == "b", mode("abcbdb")
assert mode("fe fi fo fum fi fi".split()) == "fi"
# Since Python 3.8 a tie returns the first-seen value rather than erroring.
assert mode([1, 1, 2, 2, 3]) in (1, 2), mode([1, 1, 2, 2, 3])
# A Counter is iterated by key, so the first key wins on a count tie.
assert mode(collections.Counter(a=1, b=2)) == "a"

print("mode_single_most_common OK")

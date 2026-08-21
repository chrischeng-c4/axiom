# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "sequence_matches_list_tuple_range_array_memoryview"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a sequence pattern matches list, tuple, range, array and memoryview"""

import array


# A sequence pattern matches list, tuple, range, array and memoryview.
def first_last(seq):
    match seq:
        case [head, *_, tail]:
            return (head, tail)
        case [only]:
            return ("single", only)
        case []:
            return "empty"
    return "no-match"


assert first_last([10, 20, 30]) == (10, 30)
assert first_last((10, 20, 30)) == (10, 30)
assert first_last(range(3)) == (0, 2)
assert first_last(array.array("b", b"abc")) == (97, 99)
assert first_last(memoryview(b"abc")) == (97, 99)
assert first_last(()) == "empty"
assert first_last([5]) == ("single", 5)
print("sequence_matches_list_tuple_range_array_memoryview OK")

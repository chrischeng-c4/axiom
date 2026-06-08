# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "iterator_not_matched_left_unconsumed"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: an iterator is not matched by a sequence pattern and is left unconsumed"""

# An iterator is not matched by a sequence pattern, and is left unconsumed.
it = iter([1, 2, 3])
matched_seq = False
match it:
    case []:
        matched_seq = True
assert matched_seq is False
assert list(it) == [1, 2, 3]
print("iterator_not_matched_left_unconsumed OK")

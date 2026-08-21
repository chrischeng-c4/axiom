# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "fixed_length_sequence_requires_exact_count"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a fixed-length sequence pattern requires an exact element count"""

# A fixed-length sequence pattern requires an exact element count.
def triple(seq):
    match seq:
        case [a, b, c]:
            return (a, b, c)
    return None


assert triple((1, 2, 3)) == (1, 2, 3)
assert triple((1, 2)) is None
assert triple((1, 2, 3, 4)) is None
print("fixed_length_sequence_requires_exact_count OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "str_and_bytes_not_sequence_patterns"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: str and bytes are not treated as sequence patterns"""

# str and bytes are NOT treated as sequence patterns.
def as_seq(seq):
    match seq:
        case [head, *_, tail]:
            return (head, tail)
    return "no-match"


assert as_seq("abc") == "no-match"
assert as_seq(b"abc") == "no-match"
assert as_seq([1, 2, 3]) == (1, 3)  # a real sequence still matches
print("str_and_bytes_not_sequence_patterns OK")

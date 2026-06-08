# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "isterminal_partitions_at_nt_offset"
# subject = "token.ISTERMINAL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.ISTERMINAL: ISTERMINAL is True below NT_OFFSET and False at/above it, the exact complement of ISNONTERMINAL"""
import token

# NT_OFFSET (256) is the boundary between terminal and non-terminal types.
assert token.NT_OFFSET == 256, token.NT_OFFSET

# Below the offset: terminal tokens. At/above: non-terminals.
for tok_type in [0, 1, token.NAME, token.NT_OFFSET - 1]:
    assert token.ISTERMINAL(tok_type) is True, tok_type
    assert token.ISNONTERMINAL(tok_type) is False, tok_type

for tok_type in [token.NT_OFFSET, token.NT_OFFSET + 1, 300]:
    assert token.ISTERMINAL(tok_type) is False, tok_type
    assert token.ISNONTERMINAL(tok_type) is True, tok_type

# The two predicates are exact complements across the boundary.
for tok_type in [0, 1, 255, 256, 257]:
    assert token.ISTERMINAL(tok_type) != token.ISNONTERMINAL(tok_type), tok_type

print("isterminal_partitions_at_nt_offset OK")

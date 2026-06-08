# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: boolean-algebra identities over two concrete sets."""

a = set("abracadabra")   # {a, b, r, c, d}
b = set("alacazam")      # {a, l, c, z, m}
empty = set()

# Commutativity of the symmetric operators.
assert a & b == b & a
assert a | b == b | a
assert a ^ b == b ^ a
# Difference is not commutative for distinct sets.
assert a != b
assert a - b != b - a

# Subset relationships between binops and operands.
assert a - b < a
assert b - a < b
assert a & b < a
assert a & b < b
assert a | b > a
assert a | b > b
assert a ^ b < a | b

# Mutual exclusion: a difference shares nothing with the other operand.
assert (a - b) & b == empty
assert (b - a) & a == empty
assert a & b & (a ^ b) == empty

# Summation identities reconstructing the union.
assert (a - b) | (a & b) | (b - a) == a | b
assert (a & b) | (a ^ b) == a | b
assert a | (b - a) == a | b
assert (a - b) | b == a | b
assert (a - b) | (a & b) == a
assert (b - a) | (a & b) == b
assert (a - b) | (b - a) == a ^ b

# De Morgan-style relation between xor and the two differences.
assert a ^ b == (a - b) | (b - a)

print("algebra_identities OK")

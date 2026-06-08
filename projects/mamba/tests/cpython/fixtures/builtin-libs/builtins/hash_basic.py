# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# hash() across primitive types.
# Regression: hash(non-integral float) used to panic in from_int because the
# f64 bit pattern has the NaN prefix set, exceeding the 48-bit int payload.

# Integers: hash(n) == n, except hash(-1) remaps to -2.
print(hash(0))
print(hash(1))
print(hash(-1))
print(hash(42))
print(hash(-42))

# Integral floats: hash(n.0) == hash(n).
print(hash(0.0) == hash(0))
print(hash(1.0) == hash(1))
print(hash(-1.0) == hash(-1))
print(hash(42.0) == hash(42))

# Bools are ints.
print(hash(True))
print(hash(False))

# None hashes to a fixed value.
print(hash(None) == hash(None))

# Non-integral floats: hash exists and is stable within the process.
h1 = hash(1.5)
h2 = hash(1.5)
print(h1 == h2)
print(type(h1).__name__)

# Different non-integral floats produce different hashes (collision is legal
# but these two specific values don't collide in CPython or mamba).
print(hash(1.5) != hash(2.5))

# Strings / tuples have their own hash paths — verify stability.
print(hash("") == hash(""))
print(hash("abc") == hash("abc"))
print(hash((1, 2, 3)) == hash((1, 2, 3)))

# Non-integral floats in a dict: lookup must succeed.
d = {}
d[1.5] = "one-point-five"
d[2.25] = "two-point-two-five"
print(d[1.5])
print(d[2.25])
print(len(d))

# Non-integral floats in a set: membership must work.
s = {0.5, 1.5, 2.5}
print(1.5 in s)
print(3.5 in s)
print(len(s))

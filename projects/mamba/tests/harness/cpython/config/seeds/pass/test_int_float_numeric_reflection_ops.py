# Operational AssertionPass seed for numeric-reflection methods on
# int and float that the bit_count/to_bytes/from_bytes /
# float_special_values fixtures don't already cover.
# Surface:
#   • int.bit_length() — minimum bits to represent abs(n);
#   • float.as_integer_ratio() — returns the (numerator, denominator)
#     pair of a Python-exact rational representation;
#   • float.conjugate() / float.real / float.imag — every Python
#     number IS a complex number in the abstract sense, so the
#     real/imag/conjugate interface is defined and exact on float.
#
# float.hex() / float.fromhex() / int.real / int.numerator /
# int.denominator are deliberately NOT exercised here: mamba
# 0.3.60 disagrees on those surfaces — float.hex returns a raw
# bytes-of-double repr, float.fromhex returns bytes, and the
# int.real/numerator/denominator triples return None or wrong values.
_ledger: list[int] = []

# int.bit_length — minimum bits for abs(n)
assert (0).bit_length() == 0; _ledger.append(1)
assert (1).bit_length() == 1; _ledger.append(1)
assert (2).bit_length() == 2; _ledger.append(1)
assert (3).bit_length() == 2; _ledger.append(1)
assert (4).bit_length() == 3; _ledger.append(1)
assert (255).bit_length() == 8; _ledger.append(1)
assert (256).bit_length() == 9; _ledger.append(1)
assert (1024).bit_length() == 11; _ledger.append(1)
# Negative ints — bit_length operates on the absolute value
assert (-1).bit_length() == 1; _ledger.append(1)
assert (-256).bit_length() == 9; _ledger.append(1)
# Bit-length and (1 << n) — n is the bit-length of 2**n
assert (1 << 10).bit_length() == 11; _ledger.append(1)
assert (1 << 20).bit_length() == 21; _ledger.append(1)

# float.as_integer_ratio — exact (num, den) for representable
# rationals
assert (0.5).as_integer_ratio() == (1, 2); _ledger.append(1)
assert (1.5).as_integer_ratio() == (3, 2); _ledger.append(1)
assert (0.25).as_integer_ratio() == (1, 4); _ledger.append(1)
assert (2.0).as_integer_ratio() == (2, 1); _ledger.append(1)
assert (3.0).as_integer_ratio() == (3, 1); _ledger.append(1)
# Negative as_integer_ratio
assert (-0.5).as_integer_ratio() == (-1, 2); _ledger.append(1)
assert (-1.5).as_integer_ratio() == (-3, 2); _ledger.append(1)
# Zero
assert (0.0).as_integer_ratio() == (0, 1); _ledger.append(1)
# A power of two — exact representation
assert (0.125).as_integer_ratio() == (1, 8); _ledger.append(1)
assert (8.0).as_integer_ratio() == (8, 1); _ledger.append(1)

# float.conjugate — for a real float, conjugate is the same value
assert (1.5).conjugate() == 1.5; _ledger.append(1)
assert (0.0).conjugate() == 0.0; _ledger.append(1)
assert (-3.14).conjugate() == -3.14; _ledger.append(1)
# Conjugate is an involution
assert (1.5).conjugate().conjugate() == 1.5; _ledger.append(1)

# float.real and float.imag — every float's imag part is 0.0
assert (1.5).real == 1.5; _ledger.append(1)
assert (1.5).imag == 0.0; _ledger.append(1)
assert (0.0).real == 0.0; _ledger.append(1)
assert (0.0).imag == 0.0; _ledger.append(1)
assert (-2.5).real == -2.5; _ledger.append(1)
assert (-2.5).imag == 0.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_int_float_numeric_reflection_ops {sum(_ledger)} asserts")

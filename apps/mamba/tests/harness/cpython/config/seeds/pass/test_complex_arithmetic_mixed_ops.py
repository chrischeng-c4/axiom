# Operational AssertionPass seed for `complex` mixed-type arithmetic.
# Surface:
#   • complex op int, complex op float (add/sub/mul/div all preserve
#     complex type and distribute over real/imag parts);
#   • complex literal `Nj` and `complex(re, im)` / `complex(re)` /
#     `complex(str)` / no-arg `complex()` constructors;
#   • abs(complex) — Euclidean modulus including zero and the (-3,4)
#     case (sign of the real part is squared away);
#   • .real / .imag attribute reads as float;
#   • complex.conjugate() — flips imag sign;
#   • complex == complex with same / different components;
#   • unary +/- on complex;
#   • int and float promote to complex in mixed arithmetic
#     (`(3+4j)+2 == (5+4j)`);
#   • complex ** int small-power dispatch (squared, identity, zeroth);
#   • hash(complex) — equal complexes hash equal;
#   • `1j * 1j == -1+0j` — fundamental imaginary identity.
#
# Equality with a bare int (`complex(0,0) == 0`) is deliberately NOT
# asserted — mamba 0.3.60 returns False where CPython returns True via
# int→complex coercion; that gap moves to a focused spec/ seed.
_ledger: list[int] = []

c = complex(3, 4)

# complex op int
assert (c + 2) == complex(5, 4); _ledger.append(1)
assert (c - 1) == complex(2, 4); _ledger.append(1)
assert (c * 3) == complex(9, 12); _ledger.append(1)
assert (c / 2) == complex(1.5, 2); _ledger.append(1)

# complex op float
assert (c + 1.5) == complex(4.5, 4); _ledger.append(1)
assert (c * 1.5) == complex(4.5, 6); _ledger.append(1)

# complex op complex
assert (c - complex(1, 2)) == complex(2, 2); _ledger.append(1)
assert (c * complex(2, 0)) == complex(6, 8); _ledger.append(1)
assert (c + complex(1, 2)) == complex(4, 6); _ledger.append(1)

# complex ** int
assert (c ** 2) == complex(-7, 24); _ledger.append(1)
assert (c ** 0) == complex(1, 0); _ledger.append(1)
assert (c ** 1) == c; _ledger.append(1)

# unary +/-
assert (-c) == complex(-3, -4); _ledger.append(1)
assert (+c) == c; _ledger.append(1)

# abs() — Euclidean modulus
assert abs(c) == 5.0; _ledger.append(1)
assert abs(complex(0, 0)) == 0.0; _ledger.append(1)
assert abs(complex(-3, 4)) == 5.0; _ledger.append(1)
assert abs(complex(0, 1)) == 1.0; _ledger.append(1)

# real / imag attribute reads
assert complex(3, 0).real == 3.0; _ledger.append(1)
assert complex(0, 3).imag == 3.0; _ledger.append(1)
assert c.real == 3.0; _ledger.append(1)
assert c.imag == 4.0; _ledger.append(1)

# conjugate
assert c.conjugate() == complex(3, -4); _ledger.append(1)
assert complex(0, 1).conjugate() == complex(0, -1); _ledger.append(1)
assert c.conjugate().conjugate() == c; _ledger.append(1)  # involution

# equality
assert complex(0, 0) == complex(0, 0); _ledger.append(1)
assert complex(3, 4) == complex(3, 4); _ledger.append(1)
assert complex(3, 4) != complex(3, 5); _ledger.append(1)

# constructors
assert complex(1.5) == complex(1.5, 0); _ledger.append(1)
assert complex("1+2j") == complex(1, 2); _ledger.append(1)
assert complex() == complex(0, 0); _ledger.append(1)

# hash
assert hash(complex(3, 4)) == hash(complex(3, 4)); _ledger.append(1)

# fundamental imaginary identity: i*i == -1
assert 1j * 1j == complex(-1, 0); _ledger.append(1)

# literal form
assert (3 + 4j) == complex(3, 4); _ledger.append(1)
assert type(3 + 4j).__name__ == "complex"; _ledger.append(1)
assert (0 + 1j).imag == 1.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_complex_arithmetic_mixed_ops {sum(_ledger)} asserts")

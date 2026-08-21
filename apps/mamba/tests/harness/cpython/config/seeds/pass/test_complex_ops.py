# Operational AssertionPass seed for built-in `complex` numbers.
# Surface: complex literal `3 + 4j`, real/imag attributes, abs() as
# Euclidean modulus, arithmetic (add, mul), conjugate(),
# complex(re, im) and complex("re+imj") constructors, equality.
_ledger: list[int] = []

a = 3 + 4j
# real and imag are float-typed attribute reads on the complex
assert a.real == 3.0; _ledger.append(1)
assert a.imag == 4.0; _ledger.append(1)
# abs() of a complex is its Euclidean modulus (sqrt(3² + 4²) = 5)
assert abs(a) == 5.0; _ledger.append(1)
# Adding an imaginary literal shifts imag only
assert a + 1j == complex(3, 5); _ledger.append(1)
# Scalar multiply distributes over re/imag
assert a * 2 == complex(6, 8); _ledger.append(1)
# Conjugate flips the imag sign
assert a.conjugate() == complex(3, -4); _ledger.append(1)
# Two-argument complex() constructor
assert complex(1, 2) == 1 + 2j; _ledger.append(1)
# String parsing on the complex constructor
assert complex("3+4j") == 3 + 4j; _ledger.append(1)
# Equality is component-wise and tolerates int vs float comparison
assert a == complex(3, 4); _ledger.append(1)
# Self-equality (identity-style check, value-equal)
assert a == a; _ledger.append(1)
# Pure-imaginary literal
assert (0 + 1j).imag == 1.0; _ledger.append(1)
# Pure-real complex equals an int after .real read
assert (5 + 0j).real == 5.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_complex_ops {sum(_ledger)} asserts")

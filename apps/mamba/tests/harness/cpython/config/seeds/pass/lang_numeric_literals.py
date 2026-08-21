# Operational AssertionPass seed for numeric literal forms.
# Surface: decimal int; hex (0x / 0X), octal (0o / 0O), binary (0b /
# 0B); underscore digit separators in int, hex, and binary literals;
# float — explicit (1.5), leading-dot (.5), trailing-dot (5.); scien-
# tific notation (e / E, with + or - exponent); negative integer
# and float; integer power producing a 32-bit-spanning value (2**32);
# True/False/None literal singletons; True == 1 and False == 0 under
# Python's bool-int subclassing; complex literal (1j) and complex()
# constructor.
_ledger: list[int] = []

# Decimal integer literal
assert 123 == 123; _ledger.append(1)

# Hexadecimal (0x / 0X)
assert 0xff == 255; _ledger.append(1)
assert 0XFF == 255; _ledger.append(1)
assert 0x0 == 0; _ledger.append(1)

# Octal (0o / 0O) — base 8
assert 0o17 == 15; _ledger.append(1)
assert 0o775 == 509; _ledger.append(1)
assert 0o0 == 0; _ledger.append(1)

# Binary (0b / 0B)
assert 0b101 == 5; _ledger.append(1)
assert 0B101 == 5; _ledger.append(1)
assert 0b0 == 0; _ledger.append(1)

# Underscore digit separators on decimal and hex / binary / octal
assert 1_000 == 1000; _ledger.append(1)
assert 1_000_000 == 1000000; _ledger.append(1)
assert 0xff_ff == 65535; _ledger.append(1)
assert 0b1010_1010 == 170; _ledger.append(1)
assert 0b1_0000_0000 == 256; _ledger.append(1)
assert 0o7_7_7 == 511; _ledger.append(1)

# Float literal forms — explicit, leading-dot, trailing-dot
assert 1.5 == 1.5; _ledger.append(1)
assert .5 == 0.5; _ledger.append(1)
assert 5. == 5.0; _ledger.append(1)
assert 3.14 == 3.14; _ledger.append(1)
assert 0.0 == 0.0; _ledger.append(1)

# Scientific notation
assert 1e3 == 1000.0; _ledger.append(1)
assert 1E3 == 1000.0; _ledger.append(1)
assert 1e+10 == 10000000000.0; _ledger.append(1)
assert 1e-3 == 0.001; _ledger.append(1)
assert 2.5e2 == 250.0; _ledger.append(1)

# Negative literals
assert -5 == -5; _ledger.append(1)
assert -0.5 == -0.5; _ledger.append(1)
assert -3.14 == -3.14; _ledger.append(1)

# Integer power producing a 32-bit-spanning value
assert 10 ** 6 == 1000000; _ledger.append(1)
assert 2 ** 32 == 4294967296; _ledger.append(1)

# Bool / None singleton literals
assert True == True; _ledger.append(1)
assert False == False; _ledger.append(1)
assert None is None; _ledger.append(1)

# Bool is a subclass of int — True == 1, False == 0
assert True == 1; _ledger.append(1)
assert False == 0; _ledger.append(1)
# Arithmetic between bool and int promotes to int
assert (True + True) == 2; _ledger.append(1)
assert (False + 5) == 5; _ledger.append(1)
assert (True * 10) == 10; _ledger.append(1)

# Complex literal — j suffix, real / imag components
assert 1j == 1j; _ledger.append(1)
assert (3 + 4j).real == 3.0; _ledger.append(1)
assert (3 + 4j).imag == 4.0; _ledger.append(1)
assert (2j + 3) == complex(3, 2); _ledger.append(1)

# Hex round-trip
assert 0xFF == 0xff; _ledger.append(1)
assert 0xCAFE == 51966; _ledger.append(1)

# Boundary integer values
assert 0xFFFF == 65535; _ledger.append(1)
assert 0xFFFF_FFFF == 4294967295; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_numeric_literals {sum(_ledger)} asserts")

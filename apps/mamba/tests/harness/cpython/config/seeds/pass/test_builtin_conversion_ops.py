# Operational AssertionPass seed for built-in number/string conversion
# and arithmetic functions.
# Surface: bin/oct/hex base prefixes, ord/chr inverse pair (incl. a
# non-ASCII codepoint), divmod, abs (int + float), pow (2-arg and
# 3-arg modular), round (banker's rounding + precision), repr,
# ascii (non-ASCII escapes).
_ledger: list[int] = []
# Base prefix + value
assert bin(10) == "0b1010"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert hex(255) == "0xff"; _ledger.append(1)
# ord / chr inverse on ASCII
assert ord("A") == 65; _ledger.append(1)
assert chr(65) == "A"; _ledger.append(1)
# Round-trip across the BMP
assert chr(0x4E2D) == "中"; _ledger.append(1)
assert ord("中") == 0x4E2D; _ledger.append(1)
# divmod returns (quotient, remainder)
q, r = divmod(17, 5)
assert q == 3; _ledger.append(1)
assert r == 2; _ledger.append(1)
# abs over int and float
assert abs(-7) == 7; _ledger.append(1)
assert abs(-3.14) == 3.14; _ledger.append(1)
# pow with 2 and 3 args
assert pow(2, 10) == 1024; _ledger.append(1)
assert pow(3, 4, 5) == 1; _ledger.append(1)
# round: half-to-even (banker's rounding)
assert round(3.5) == 4; _ledger.append(1)
assert round(2.5) == 2; _ledger.append(1)
# round with explicit precision
assert round(3.14159, 2) == 3.14; _ledger.append(1)
# repr surface
assert repr("hello") == "'hello'"; _ledger.append(1)
assert repr([1, 2]) == "[1, 2]"; _ledger.append(1)
# ascii() emits \xNN for non-ASCII
assert ascii("café") == "'caf\\xe9'"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_builtin_conversion_ops {sum(_ledger)} asserts")

# Operational AssertionPass seed for character-and-integer conversion
# builtins.
# Surface: chr(int) returns the single-character str for a codepoint;
# ord(str) returns the int codepoint of a one-char string; chr and ord
# are inverses on the ASCII range; hex / oct / bin format ints with
# their standard prefixes (0x, 0o, 0b) and handle zero and negative
# inputs; repr returns the standard textual representation for str,
# int, and list.
_ledger: list[int] = []

# chr(int) returns the single-character str for that codepoint
assert chr(65) == "A"; _ledger.append(1)
assert chr(97) == "a"; _ledger.append(1)
assert chr(48) == "0"; _ledger.append(1)
assert chr(32) == " "; _ledger.append(1)

# ord(str) returns the integer codepoint of a one-character string
assert ord("A") == 65; _ledger.append(1)
assert ord("a") == 97; _ledger.append(1)
assert ord("0") == 48; _ledger.append(1)
assert ord(" ") == 32; _ledger.append(1)

# chr and ord round-trip across the ASCII range
assert chr(ord("Z")) == "Z"; _ledger.append(1)
assert ord(chr(100)) == 100; _ledger.append(1)

# hex(int) formats with the 0x prefix
assert hex(255) == "0xff"; _ledger.append(1)
assert hex(0) == "0x0"; _ledger.append(1)
assert hex(16) == "0x10"; _ledger.append(1)
# Negative ints get a leading minus sign before the prefix
assert hex(-16) == "-0x10"; _ledger.append(1)

# oct(int) formats with the 0o prefix
assert oct(8) == "0o10"; _ledger.append(1)
assert oct(64) == "0o100"; _ledger.append(1)
assert oct(0) == "0o0"; _ledger.append(1)

# bin(int) formats with the 0b prefix
assert bin(10) == "0b1010"; _ledger.append(1)
assert bin(0) == "0b0"; _ledger.append(1)
assert bin(255) == "0b11111111"; _ledger.append(1)
# Negative binary picks up the leading minus
assert bin(-5) == "-0b101"; _ledger.append(1)

# repr(int) gives the integer's decimal text representation
assert repr(42) == "42"; _ledger.append(1)
assert repr(0) == "0"; _ledger.append(1)

# repr(str) wraps the string in quotes
assert repr("hello") == "'hello'"; _ledger.append(1)
assert repr("") == "''"; _ledger.append(1)

# repr(list) renders [item1, item2, ...] with comma-space separators
assert repr([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert repr([]) == "[]"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_chr_ord_hex_repr_ops {sum(_ledger)} asserts")

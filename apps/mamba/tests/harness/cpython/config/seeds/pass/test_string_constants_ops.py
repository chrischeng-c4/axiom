# Operational AssertionPass seed for the `string` constants module.
# Surface: ascii_lowercase, ascii_uppercase, ascii_letters, digits,
# hexdigits, octdigits, plus structural invariants (length, ordering,
# containment).
import string
_ledger: list[int] = []
# Canonical alphabets
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
# ascii_letters is the concatenation
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase; _ledger.append(1)
assert len(string.ascii_letters) == 52; _ledger.append(1)
# Digit alphabets
assert string.digits == "0123456789"; _ledger.append(1)
# hexdigits include 0-9 + a-f + A-F
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert len(string.hexdigits) == 22; _ledger.append(1)
# Containment invariants
assert "a" in string.ascii_lowercase; _ledger.append(1)
assert "A" in string.ascii_uppercase; _ledger.append(1)
assert "5" in string.digits; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_string_constants_ops {sum(_ledger)} asserts")

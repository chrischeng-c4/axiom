# Operational AssertionPass seed for the `string` stdlib module.
# Surface: ascii_lowercase/uppercase/letters, digits, hexdigits,
# octdigits, punctuation, whitespace constants.
# Companion to stub/test_string.py — vendored unittest seed.
import string
_ledger: list[int] = []
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert "a" in string.ascii_lowercase; _ledger.append(1)
assert "Z" in string.ascii_uppercase; _ledger.append(1)
assert "5" in string.digits; _ledger.append(1)
assert "!" in string.punctuation; _ledger.append(1)
assert " " in string.whitespace; _ledger.append(1)
assert len(string.ascii_lowercase) == 26; _ledger.append(1)
assert len(string.ascii_uppercase) == 26; _ledger.append(1)
assert len(string.digits) == 10; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_string_module_ops {sum(_ledger)} asserts")

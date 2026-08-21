# Operational AssertionPass seed for the matching `string` module
# surface — character-class constants + capwords on whitespace-
# separated input. There is no existing string-module seed.
#
# `string` defines a handful of ASCII character-class constants
# (ascii_lowercase, ascii_uppercase, ascii_letters, digits, hexdigits,
# octdigits, punctuation, whitespace) and a small set of helpers
# (capwords, Template, Formatter). The constants and the
# default-whitespace `capwords` form constitute the stable matching
# subset against mamba.
#
# Surface in this fixture:
#   • ascii_lowercase / ascii_uppercase / ascii_letters — exact char
#     content + length;
#   • digits / hexdigits / octdigits — exact char content + length +
#     membership;
#   • punctuation — character-class membership (it/not it, with the
#     non-printable letters/digits explicitly NOT in punctuation);
#   • whitespace — character-class membership (space, tab, newline,
#     carriage return ARE whitespace; ASCII letters are NOT);
#   • capwords with default whitespace separator — capitalize-first-
#     letter-of-each-word semantics, including: two-word, three-word,
#     mixed-case (recapitalize), tab/newline separators, empty string,
#     single-word, already-capitalized, and leading/trailing
#     whitespace inputs.
#
# Behavioral edges that DIVERGE on mamba (printable, capwords with a
# custom sep, Template substitution, Formatter().format) are covered
# in `lang_string_template_formatter_silent.py`.
import string

_ledger: list[int] = []

# 1) ascii_lowercase / ascii_uppercase / ascii_letters — exact content
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase; _ledger.append(1)
assert len(string.ascii_lowercase) == 26; _ledger.append(1)
assert len(string.ascii_uppercase) == 26; _ledger.append(1)
assert len(string.ascii_letters) == 52; _ledger.append(1)

# Per-char membership
assert "a" in string.ascii_lowercase; _ledger.append(1)
assert "z" in string.ascii_lowercase; _ledger.append(1)
assert "A" not in string.ascii_lowercase; _ledger.append(1)
assert "A" in string.ascii_uppercase; _ledger.append(1)
assert "Z" in string.ascii_uppercase; _ledger.append(1)
assert "a" not in string.ascii_uppercase; _ledger.append(1)
assert "a" in string.ascii_letters; _ledger.append(1)
assert "Z" in string.ascii_letters; _ledger.append(1)
assert "0" not in string.ascii_letters; _ledger.append(1)

# 2) digits / hexdigits / octdigits — exact content + length
assert string.digits == "0123456789"; _ledger.append(1)
assert len(string.digits) == 10; _ledger.append(1)
assert "0" in string.digits; _ledger.append(1)
assert "9" in string.digits; _ledger.append(1)
assert "a" not in string.digits; _ledger.append(1)

assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert len(string.hexdigits) == 22; _ledger.append(1)
assert "a" in string.hexdigits; _ledger.append(1)
assert "F" in string.hexdigits; _ledger.append(1)
assert "g" not in string.hexdigits; _ledger.append(1)
assert "G" not in string.hexdigits; _ledger.append(1)

assert string.octdigits == "01234567"; _ledger.append(1)
assert len(string.octdigits) == 8; _ledger.append(1)
assert "7" in string.octdigits; _ledger.append(1)
assert "8" not in string.octdigits; _ledger.append(1)

# 3) punctuation — character-class membership
#    (exact content varies in repr; test by membership)
assert "!" in string.punctuation; _ledger.append(1)
assert "." in string.punctuation; _ledger.append(1)
assert "," in string.punctuation; _ledger.append(1)
assert "?" in string.punctuation; _ledger.append(1)
assert ":" in string.punctuation; _ledger.append(1)
assert ";" in string.punctuation; _ledger.append(1)
assert "@" in string.punctuation; _ledger.append(1)
assert "#" in string.punctuation; _ledger.append(1)
assert "(" in string.punctuation; _ledger.append(1)
assert ")" in string.punctuation; _ledger.append(1)
# Letters / digits are NOT punctuation
assert "a" not in string.punctuation; _ledger.append(1)
assert "Z" not in string.punctuation; _ledger.append(1)
assert "0" not in string.punctuation; _ledger.append(1)

# 4) whitespace — character-class membership
assert " " in string.whitespace; _ledger.append(1)
assert "\t" in string.whitespace; _ledger.append(1)
assert "\n" in string.whitespace; _ledger.append(1)
assert "\r" in string.whitespace; _ledger.append(1)
# ASCII letters and digits are NOT whitespace
assert "a" not in string.whitespace; _ledger.append(1)
assert "A" not in string.whitespace; _ledger.append(1)
assert "0" not in string.whitespace; _ledger.append(1)
assert "z" not in string.whitespace; _ledger.append(1)

# 5) capwords with default whitespace separator
#    two-word
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)
#    three-word
assert string.capwords("the quick brown fox") == "The Quick Brown Fox"; _ledger.append(1)
#    mixed-case input is recapitalized
assert string.capwords("HELLO world FOO bar") == "Hello World Foo Bar"; _ledger.append(1)
#    tab and newline separators
assert string.capwords("hello\tworld") == "Hello World"; _ledger.append(1)
assert string.capwords("hello\nworld") == "Hello World"; _ledger.append(1)
#    empty / single-word
assert string.capwords("") == ""; _ledger.append(1)
assert string.capwords("hello") == "Hello"; _ledger.append(1)
#    already capitalized stays capitalized
assert string.capwords("Hello World") == "Hello World"; _ledger.append(1)
#    multi-space collapse / leading-trailing whitespace
assert string.capwords("hello  world") == "Hello World"; _ledger.append(1)
assert string.capwords(" hello world") == "Hello World"; _ledger.append(1)
assert string.capwords("hello world ") == "Hello World"; _ledger.append(1)

# 6) Constants are str type
assert isinstance(string.ascii_letters, str); _ledger.append(1)
assert isinstance(string.digits, str); _ledger.append(1)
assert isinstance(string.hexdigits, str); _ledger.append(1)
assert isinstance(string.whitespace, str); _ledger.append(1)
assert isinstance(string.punctuation, str); _ledger.append(1)

# 7) Cross-constant composition (constants concatenate to predictable
#    sizes — useful for table-driven char classification)
assert len(string.ascii_lowercase + string.ascii_uppercase + string.digits) == 62; _ledger.append(1)
# Letters + digits == 62 chars (52 + 10)
assert len(string.ascii_letters + string.digits) == 62; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string_constants_capwords_ops {sum(_ledger)} asserts")

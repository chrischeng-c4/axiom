# Operational AssertionPass seed for the `string` module — the
# stdlib character-class constants and the `capwords` text helper.
# Used by tokenizers, parsers, identifier validators, password-
# generator picksets (`ascii_letters + digits`), case normalizers,
# and any code that needs the canonical alphabet / digit / hex /
# punctuation / whitespace tables. Surface focuses on the constants
# (which are pure str data, identical across runtimes) and on
# `capwords()` with its default whitespace separator (mamba and
# CPython agree). The `Template` class is broken on mamba (returns
# a dict instead of a Template instance) and `printable` is None
# on mamba — both excluded. Mamba's `capwords(s, sep)` with an
# explicit separator lowercases the tail of split tokens
# differently from CPython — also excluded. No fixture coverage
# yet for the string module.
#
# Surface (matching subset):
#   • string.ascii_lowercase   = 'abc...z'      — 26 chars
#   • string.ascii_uppercase   = 'ABC...Z'      — 26 chars
#   • string.ascii_letters     = lower + upper  — 52 chars
#   • string.digits            = '0123456789'   — 10 chars
#   • string.hexdigits         = digits + 'abcdefABCDEF' — 22 chars
#   • string.octdigits         = '01234567'     — 8 chars
#   • string.punctuation       = !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~ — 32 chars
#   • string.whitespace        = ' \t\n\r\x0b\x0c'   — 6 chars
#   • string.capwords(s)       — title-case each word (whitespace
#                                splits), collapses runs of whitespace
#                                to a single space.
import string
_ledger: list[int] = []

# ascii_lowercase — exact content
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert len(string.ascii_lowercase) == 26; _ledger.append(1)
assert "a" in string.ascii_lowercase; _ledger.append(1)
assert "z" in string.ascii_lowercase; _ledger.append(1)
assert "A" not in string.ascii_lowercase; _ledger.append(1)

# ascii_uppercase — exact content
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert len(string.ascii_uppercase) == 26; _ledger.append(1)
assert "A" in string.ascii_uppercase; _ledger.append(1)
assert "Z" in string.ascii_uppercase; _ledger.append(1)
assert "a" not in string.ascii_uppercase; _ledger.append(1)

# ascii_letters — lower + upper
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase; _ledger.append(1)
assert len(string.ascii_letters) == 52; _ledger.append(1)
assert "a" in string.ascii_letters; _ledger.append(1)
assert "Z" in string.ascii_letters; _ledger.append(1)
assert "0" not in string.ascii_letters; _ledger.append(1)

# digits — exact content
assert string.digits == "0123456789"; _ledger.append(1)
assert len(string.digits) == 10; _ledger.append(1)
assert "0" in string.digits; _ledger.append(1)
assert "9" in string.digits; _ledger.append(1)
assert "a" not in string.digits; _ledger.append(1)

# hexdigits — digits + a-f + A-F
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert len(string.hexdigits) == 22; _ledger.append(1)
assert "0" in string.hexdigits; _ledger.append(1)
assert "f" in string.hexdigits; _ledger.append(1)
assert "F" in string.hexdigits; _ledger.append(1)
assert "g" not in string.hexdigits; _ledger.append(1)
assert "G" not in string.hexdigits; _ledger.append(1)

# octdigits — exact content
assert string.octdigits == "01234567"; _ledger.append(1)
assert len(string.octdigits) == 8; _ledger.append(1)
assert "0" in string.octdigits; _ledger.append(1)
assert "7" in string.octdigits; _ledger.append(1)
assert "8" not in string.octdigits; _ledger.append(1)

# punctuation — content membership (not exact, depends on order assumption)
assert len(string.punctuation) == 32; _ledger.append(1)
for _p in "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~":
    assert _p in string.punctuation; _ledger.append(1)
assert "a" not in string.punctuation; _ledger.append(1)
assert "0" not in string.punctuation; _ledger.append(1)

# whitespace — exact content (chars are identical even if repr differs)
assert len(string.whitespace) == 6; _ledger.append(1)
assert " " in string.whitespace; _ledger.append(1)
assert "\t" in string.whitespace; _ledger.append(1)
assert "\n" in string.whitespace; _ledger.append(1)
assert "\r" in string.whitespace; _ledger.append(1)
assert chr(11) in string.whitespace; _ledger.append(1)
assert chr(12) in string.whitespace; _ledger.append(1)
assert "a" not in string.whitespace; _ledger.append(1)
assert string.whitespace == " \t\n\r\x0b\x0c"; _ledger.append(1)

# capwords — default whitespace splitter
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)
assert string.capwords("a b c") == "A B C"; _ledger.append(1)
assert string.capwords("HELLO WORLD") == "Hello World"; _ledger.append(1)
assert string.capwords("hello") == "Hello"; _ledger.append(1)
assert string.capwords("") == ""; _ledger.append(1)
# Collapses repeated whitespace
assert string.capwords("  hello  world  ") == "Hello World"; _ledger.append(1)
assert string.capwords("hello   world") == "Hello World"; _ledger.append(1)
# Multi-word
assert string.capwords("the quick brown fox") == "The Quick Brown Fox"; _ledger.append(1)

# Type discipline
assert isinstance(string.ascii_lowercase, str); _ledger.append(1)
assert isinstance(string.ascii_uppercase, str); _ledger.append(1)
assert isinstance(string.ascii_letters, str); _ledger.append(1)
assert isinstance(string.digits, str); _ledger.append(1)
assert isinstance(string.hexdigits, str); _ledger.append(1)
assert isinstance(string.octdigits, str); _ledger.append(1)
assert isinstance(string.punctuation, str); _ledger.append(1)
assert isinstance(string.whitespace, str); _ledger.append(1)
assert isinstance(string.capwords("hi"), str); _ledger.append(1)

# Module-level attribute discipline
for _name in ["ascii_lowercase", "ascii_uppercase", "ascii_letters",
              "digits", "hexdigits", "octdigits", "punctuation",
              "whitespace", "capwords"]:
    assert hasattr(string, _name); _ledger.append(1)

# Idempotence — constants don't change across reads
assert string.ascii_lowercase == string.ascii_lowercase; _ledger.append(1)
assert string.digits == string.digits; _ledger.append(1)
assert string.capwords("hi there") == string.capwords("hi there"); _ledger.append(1)

# Disjointness — characters in one class are not in mutually-exclusive ones
for _c in string.digits:
    assert _c not in string.ascii_letters; _ledger.append(1)
    assert _c not in string.punctuation; _ledger.append(1)
    assert _c not in string.whitespace; _ledger.append(1)

# ascii_letters contains all of ascii_lowercase and ascii_uppercase
for _c in string.ascii_lowercase:
    assert _c in string.ascii_letters; _ledger.append(1)
for _c in string.ascii_uppercase:
    assert _c in string.ascii_letters; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string_ascii_capwords_ops {sum(_ledger)} asserts")

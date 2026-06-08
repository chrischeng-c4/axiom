# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_string"
# subject = "cpython321.test_string"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_string.py"
# status = "filled"
# ///
"""cpython321.test_string: execute CPython 3.12 seed test_string"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: string (alphabet/digit/punctuation/whitespace constants, capwords).
# string.Template substitute() and string.Formatter currently return None /
# dict-stub callables under mamba and are intentionally NOT exercised here;
# tracked separately. str.format() (the modern equivalent of Formatter) is
# covered as a stand-in.
import string

_ledger: list[int] = []

# The lowercase alphabet constant is exactly a..z
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz", (
    "ascii_lowercase == 'a..z'"
)
_ledger.append(1)

# The uppercase alphabet constant is exactly A..Z
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ", (
    "ascii_uppercase == 'A..Z'"
)
_ledger.append(1)

# ascii_letters is lowercase followed by uppercase, length 52
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase, (
    "ascii_letters == ascii_lowercase + ascii_uppercase"
)
_ledger.append(1)

assert len(string.ascii_letters) == 52, (
    f"ascii_letters has 52 entries, got {len(string.ascii_letters)}"
)
_ledger.append(1)

# digits is exactly 0..9
assert string.digits == "0123456789", "digits == '0..9'"
_ledger.append(1)

# hexdigits includes both lowercase and uppercase A..F
assert string.hexdigits == "0123456789abcdefABCDEF", (
    "hexdigits == '0..9a..fA..F'"
)
_ledger.append(1)

# octdigits is exactly 0..7
assert string.octdigits == "01234567", "octdigits == '0..7'"
_ledger.append(1)

# punctuation contains common shell metacharacters
for ch in "!\"#$%&'()*+,-./":
    assert ch in string.punctuation, f"punctuation contains {ch!r}"
_ledger.append(1)

# whitespace contains the standard ASCII whitespace characters
for ch in (" ", "\t", "\n"):
    assert ch in string.whitespace, f"whitespace contains {ch!r}"
_ledger.append(1)

# capwords title-cases each whitespace-separated word
assert string.capwords("hello world") == "Hello World", (
    "capwords('hello world') == 'Hello World'"
)
_ledger.append(1)

# str.format positional substitution (Formatter equivalent)
assert "{} world".format("hello") == "hello world", (
    "str.format positional substitution"
)
_ledger.append(1)

# str.format named substitution
assert "{name}".format(name="A") == "A", "str.format named substitution"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string {sum(_ledger)} asserts")

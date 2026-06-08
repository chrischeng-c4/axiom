"""Behavior contract for builtins.chr and builtins.ord.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: chr maps codepoint to character
assert chr(65) == "A", f"chr(65) = {chr(65)!r}"
assert chr(97) == "a", f"chr(97) = {chr(97)!r}"
assert chr(48) == "0", f"chr(48) = {chr(48)!r}"
assert chr(32) == " ", f"chr(32) = {chr(32)!r}"
assert chr(0) == "\x00", f"chr(0) = {chr(0)!r}"

# Rule 2: chr with Unicode codepoints
assert chr(0x03B1) == "α", f"chr(0x03B1) = {chr(0x03B1)!r}"
assert chr(0x4E2D) == "中", f"chr(0x4E2D) = {chr(0x4E2D)!r}"

# Rule 3: chr out of range raises ValueError
_raised = False
try:
    chr(-1)
except ValueError:
    _raised = True
assert _raised, "chr(-1) did not raise ValueError"

_raised = False
try:
    chr(0x110000)
except ValueError:
    _raised = True
assert _raised, "chr(0x110000) did not raise ValueError"

# Rule 4: chr with non-int raises TypeError
_raised = False
try:
    chr(1.5)  # type: ignore[arg-type]
except TypeError:
    _raised = True
assert _raised, "chr(1.5) did not raise TypeError"

# Rule 5: ord maps character to codepoint
assert ord("A") == 65, f"ord('A') = {ord('A')!r}"
assert ord("a") == 97, f"ord('a') = {ord('a')!r}"
assert ord("0") == 48, f"ord('0') = {ord('0')!r}"
assert ord("\x00") == 0, f"ord('\\x00') = {ord(chr(0))!r}"

# Rule 6: ord on Unicode
assert ord("α") == 0x03B1, f"ord('α') = {ord('α')!r}"

# Rule 7: ord on empty string raises TypeError
_raised = False
try:
    ord("")
except TypeError:
    _raised = True
assert _raised, "ord('') did not raise TypeError"

# Rule 8: ord on multi-char string raises TypeError
_raised = False
try:
    ord("ab")
except TypeError:
    _raised = True
assert _raised, "ord('ab') did not raise TypeError"

# Rule 9: ord on single-byte bytes returns int (CPython 3.12 accepts bytes/bytearray)
assert ord(b"A") == 65, f"ord(b'A') = {ord(b'A')!r}"
assert ord(bytearray(b"Z")) == 90, f"ord(bytearray(b'Z')) = {ord(bytearray(b'Z'))!r}"

# multi-byte bytes raises TypeError
_raised = False
try:
    ord(b"AB")
except TypeError:
    _raised = True
assert _raised, "ord(b'AB') did not raise TypeError"

# Rule 10: chr / ord roundtrip for printable ASCII
for code in range(32, 127):
    assert ord(chr(code)) == code, f"roundtrip failed at {code}"

print("behavior OK")

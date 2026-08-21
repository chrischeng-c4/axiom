# Operational AssertionPass seed for `re` module flag surface.
# Surface: this seed asserts the flag-related behaviour that both
# CPython and mamba honour today:
#   • inline flag tokens — `(?i)`, `(?m)`, `(?s)`, `(?x)` — embedded
#     in the pattern string, which the regex compiler interprets
#     regardless of how the runtime routes the `flags` keyword
#     argument;
#   • IntFlag identity and bitwise composition on the module-level
#     constants (`re.IGNORECASE | re.MULTILINE` etc.);
#   • `re.compile(pattern, flags).flags` exposing the requested
#     flag bits masked into the compiled pattern object.
#
# The `flags=` keyword form of `re.match/search/findall/sub` is NOT
# exercised here — mamba 0.3.60 currently ignores the runtime
# `flags` argument on those callables (see
# spec/test_re_runtime_flags.py once authored). Pattern-inlined
# tokens flow through the same code path on both runtimes and are
# the portable contract.
import re
_ledger: list[int] = []

# Inline (?i) — case-insensitive match
m = re.match(r"(?i)hello", "HELLO")
assert m is not None; _ledger.append(1)
assert m.group(0) == "HELLO"; _ledger.append(1)

# Inline (?i) on a character class
m = re.match(r"(?i)[a-z]+", "ABCdef")
assert m is not None; _ledger.append(1)
assert m.group(0) == "ABCdef"; _ledger.append(1)

# Inline (?i) — mixed case literal works both directions
m = re.match(r"(?i)foo", "FoO")
assert m is not None; _ledger.append(1)

# Inline (?m) — ^ anchors at every line start
matches = re.findall(r"(?m)^foo", "foo\nbar\nfoo")
assert matches == ["foo", "foo"]; _ledger.append(1)

# Inline (?m) — $ anchors at every line end
matches = re.findall(r"(?m)foo$", "foo\nbar\nfoo")
assert matches == ["foo", "foo"]; _ledger.append(1)

# Inline (?s) — DOTALL: . matches newline
m = re.match(r"(?s)a.b", "a\nb")
assert m is not None; _ledger.append(1)
assert m.group(0) == "a\nb"; _ledger.append(1)

# Without (?s) — . does not match newline (default behaviour)
m = re.match(r"a.b", "a\nb")
assert m is None; _ledger.append(1)

# Inline (?x) — VERBOSE: whitespace + # comments ignored
pat = re.compile(r"""(?x)
    \d+        # one or more digits
    \s*        # optional whitespace
    [a-z]+     # one or more lowercase letters
""")
m = pat.match("42 abc")
assert m is not None; _ledger.append(1)
assert m.group(0) == "42 abc"; _ledger.append(1)

# Bitwise OR of flag constants — composed flag value
combined = re.IGNORECASE | re.MULTILINE
assert combined == (re.IGNORECASE | re.MULTILINE); _ledger.append(1)
assert (combined & re.IGNORECASE) == re.IGNORECASE; _ledger.append(1)
assert (combined & re.MULTILINE) == re.MULTILINE; _ledger.append(1)

# Flag constant identity / equality
assert re.IGNORECASE == re.IGNORECASE; _ledger.append(1)
assert re.IGNORECASE != re.MULTILINE; _ledger.append(1)

# re.compile(...).flags exposes the requested flag bits
pat = re.compile(r"x", re.IGNORECASE)
assert (pat.flags & re.IGNORECASE) == re.IGNORECASE; _ledger.append(1)
pat2 = re.compile(r"x", re.IGNORECASE | re.MULTILINE)
assert (pat2.flags & re.IGNORECASE) == re.IGNORECASE; _ledger.append(1)
assert (pat2.flags & re.MULTILINE) == re.MULTILINE; _ledger.append(1)

# Match-object slot surface — group / start / end
pat = re.compile(r"\d+")
m = pat.search("abc 123 def")
assert m is not None; _ledger.append(1)
assert m.group(0) == "123"; _ledger.append(1)
assert m.start() == 4; _ledger.append(1)
assert m.end() == 7; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_re_flags_ops {sum(_ledger)} asserts")

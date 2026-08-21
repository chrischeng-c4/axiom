# Operational AssertionPass seed for `bytes` method extras.
# Surface:
#   • bytes.hex() — bare form;
#   • bytes.hex(sep) — single-byte separator;
#   • bytes.hex(sep, bytes_per_sep) — separator + grouping width;
#   • bytes.hex(sep, -bytes_per_sep) — right-aligned grouping;
#   • bytes.startswith(tuple) — any-of-prefixes (also negative case);
#   • bytes.endswith(tuple) — any-of-suffixes (also negative case);
#   • bytes.strip() / .lstrip() / .rstrip() — whitespace strip;
#   • bytes.strip(b"chars") — explicit-chars strip;
#   • bytes.count(sub) / .count(sub, start) / .count(sub, start, end);
#   • bytes.find(sub) / .rfind(sub) — first / last index;
#   • bytes.replace(a, b) / .replace(a, b, count) — substitution
#     with optional max-replace cap.
#
# bytes.splitlines / .partition / .rpartition / .index / .translate
# / .maketrans / .upper / .lower / .zfill / .center are deliberately
# NOT exercised here — mamba 0.3.60 raises AttributeError on all of
# them. Each gap moves to a focused spec/ seed.
_ledger: list[int] = []

# hex — bare
assert (b"ABCD").hex() == "41424344"; _ledger.append(1)
assert (b"").hex() == ""; _ledger.append(1)
assert (b"\x00\xff").hex() == "00ff"; _ledger.append(1)

# hex(sep) — single-byte separator
assert (b"ABCD").hex("-") == "41-42-43-44"; _ledger.append(1)
assert (b"ABCD").hex(":") == "41:42:43:44"; _ledger.append(1)

# hex(sep, bytes_per_sep) — grouped
assert (b"ABCD").hex(":", 2) == "4142:4344"; _ledger.append(1)
assert (b"ABCDEF").hex(" ", 2) == "4142 4344 4546"; _ledger.append(1)
assert (b"ABCDEF").hex("-", 3) == "414243-444546"; _ledger.append(1)

# hex(sep, -bytes_per_sep) — right-aligned grouping (negative width)
assert (b"ABCDEF").hex(" ", -2) == "4142 4344 4546"; _ledger.append(1)

# startswith / endswith with tuple of bytes
assert b"hello".startswith((b"he", b"lo")); _ledger.append(1)
assert not b"hello".startswith((b"X", b"Y")); _ledger.append(1)
assert b"hello".endswith((b"he", b"lo")); _ledger.append(1)
assert not b"hello".endswith((b"X", b"Y")); _ledger.append(1)
assert b"foo.py".endswith((b".py", b".pyc")); _ledger.append(1)
assert b"abc".startswith((b"a",)); _ledger.append(1)
assert b"abc".endswith((b"c",)); _ledger.append(1)

# strip family — bare (whitespace)
assert b"  hello  ".strip() == b"hello"; _ledger.append(1)
assert b"  hello".lstrip() == b"hello"; _ledger.append(1)
assert b"hello  ".rstrip() == b"hello"; _ledger.append(1)
assert b"\t\nhello\r\n".strip() == b"hello"; _ledger.append(1)

# strip with explicit chars argument
assert b"xxhelloxx".strip(b"x") == b"hello"; _ledger.append(1)
assert b"<<data>>".strip(b"<>") == b"data"; _ledger.append(1)

# count — bare
assert b"banana".count(b"a") == 3; _ledger.append(1)
assert b"banana".count(b"n") == 2; _ledger.append(1)
assert b"banana".count(b"z") == 0; _ledger.append(1)
assert b"banana".count(b"na") == 2; _ledger.append(1)

# count with start
assert b"banana".count(b"a", 2) == 2; _ledger.append(1)
assert b"banana".count(b"n", 3) == 1; _ledger.append(1)

# count with start and end
assert b"banana".count(b"a", 2, 4) == 1; _ledger.append(1)
assert b"banana".count(b"n", 0, 3) == 1; _ledger.append(1)

# find / rfind
assert b"banana".find(b"na") == 2; _ledger.append(1)
assert b"banana".rfind(b"na") == 4; _ledger.append(1)
assert b"banana".find(b"z") == -1; _ledger.append(1)
assert b"banana".rfind(b"z") == -1; _ledger.append(1)

# replace
assert b"hello".replace(b"l", b"L") == b"heLLo"; _ledger.append(1)
assert b"hello".replace(b"l", b"L", 1) == b"heLlo"; _ledger.append(1)
assert b"hello".replace(b"l", b"L", 0) == b"hello"; _ledger.append(1)
assert b"aaaa".replace(b"a", b"b") == b"bbbb"; _ledger.append(1)
assert b"aaaa".replace(b"a", b"b", 2) == b"bbaa"; _ledger.append(1)
assert b"hello".replace(b"x", b"y") == b"hello"; _ledger.append(1)
assert b"hello world".replace(b" ", b"_") == b"hello_world"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bytes_method_extras_ops {sum(_ledger)} asserts")

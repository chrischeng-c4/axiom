# Operational AssertionPass seed for str split/partition/translate
# surfaces not covered by test_str_ops or test_str_predicates_ops.
# Surface: split(sep, maxsplit), rsplit(sep, maxsplit), partition,
# rpartition, startswith(tuple), endswith(tuple), translate +
# str.maketrans, title-case, removeprefix/removesuffix (PEP 616,
# 3.9+), rfind, index, encode/decode round-trip.
_ledger: list[int] = []

# split with explicit maxsplit caps the number of separators consumed
assert "a,b,c,d".split(",", 2) == ["a", "b", "c,d"]; _ledger.append(1)
# rsplit consumes from the right
assert "a,b,c,d".rsplit(",", 1) == ["a,b,c", "d"]; _ledger.append(1)

# partition splits into (head, sep, tail) on the first separator
assert "a,b,c".partition(",") == ("a", ",", "b,c"); _ledger.append(1)
# rpartition splits on the LAST separator
assert "a,b,c".rpartition(",") == ("a,b", ",", "c"); _ledger.append(1)
# partition returns ('input', '', '') when separator is absent
assert "abc".partition(",") == ("abc", "", ""); _ledger.append(1)

# startswith / endswith accept a tuple of candidate prefixes/suffixes
assert "http://x".startswith(("http://", "https://")); _ledger.append(1)
assert "a.tar.gz".endswith((".gz", ".zip")); _ledger.append(1)

# str.maketrans + translate replaces characters by mapping
table = str.maketrans({"a": "x", "b": "y"})
assert "abcab".translate(table) == "xycxy"; _ledger.append(1)

# title() capitalizes the first letter of each word
assert "hello world".title() == "Hello World"; _ledger.append(1)

# rfind returns the last occurrence index, or -1 when absent
assert "hello".rfind("l") == 3; _ledger.append(1)
assert "hello".rfind("z") == -1; _ledger.append(1)
# index raises if not found; here it succeeds
assert "hello".index("e") == 1; _ledger.append(1)

# PEP 616 removeprefix/removesuffix (3.9+) — non-destructive, returns
# the original when the prefix/suffix does not match
assert "pre_value".removeprefix("pre_") == "value"; _ledger.append(1)
assert "value.txt".removesuffix(".txt") == "value"; _ledger.append(1)
assert "value".removeprefix("none_") == "value"; _ledger.append(1)

# encode/decode round-trip in UTF-8
assert "hello".encode("utf-8") == b"hello"; _ledger.append(1)
assert b"hello".decode("utf-8") == "hello"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_partition_translate_ops {sum(_ledger)} asserts")

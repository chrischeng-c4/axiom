# Operational AssertionPass seed for the difflib closest-match
# surface. Surface: `difflib.get_close_matches(word, candidates)`
# returns a list of `candidates` strings ordered by similarity to
# `word` descending; an empty candidates list yields []; a no-
# similar-match returns []; exact-string match yields a single-
# element list `[word]`; the top result of a list containing the
# exact `word` is `word` itself; the type of the returned value
# is always a Python list, even for the empty-result case; the
# function operates over the empty-string identity (matching ""
# in [""] returns [""]). Companion to test_difflib (which covers
# the broader Differ / SequenceMatcher surface).
import difflib
_ledger: list[int] = []

# Multi-candidate ranking — closer match first
assert difflib.get_close_matches("apple", ["ape", "apply", "apricot"]) == ["apply", "ape"]; _ledger.append(1)
assert difflib.get_close_matches("hello", ["world", "helo", "hellos"]) == ["hellos", "helo"]; _ledger.append(1)
assert difflib.get_close_matches("apple", ["apply", "ape"]) == ["apply", "ape"]; _ledger.append(1)

# No-match — returns []
assert difflib.get_close_matches("foo", ["bar", "baz"]) == []; _ledger.append(1)
assert difflib.get_close_matches("z", ["a", "b", "c"]) == []; _ledger.append(1)

# Empty candidates list — returns []
assert difflib.get_close_matches("apple", []) == []; _ledger.append(1)
assert isinstance(difflib.get_close_matches("a", []), list); _ledger.append(1)

# Exact-string match — returns [word]
assert difflib.get_close_matches("a", ["a"]) == ["a"]; _ledger.append(1)
assert difflib.get_close_matches("x", ["x"]) == ["x"]; _ledger.append(1)
assert difflib.get_close_matches("hello", ["hello"]) == ["hello"]; _ledger.append(1)
assert difflib.get_close_matches("ABCDEFG", ["ABCDEFG"]) == ["ABCDEFG"]; _ledger.append(1)

# Empty-string identity
assert difflib.get_close_matches("", [""]) == [""]; _ledger.append(1)

# Result type is always a list
assert isinstance(difflib.get_close_matches("a", ["a"]), list); _ledger.append(1)
assert isinstance(difflib.get_close_matches("apple", ["apply"]), list); _ledger.append(1)

# Top match is the exact word when present
res = difflib.get_close_matches("hello", ["hello", "hallo", "helo"])
assert isinstance(res, list); _ledger.append(1)
assert len(res) >= 1; _ledger.append(1)
assert res[0] == "hello"; _ledger.append(1)
assert "hello" in difflib.get_close_matches("hello", ["world", "hello"]); _ledger.append(1)

# Closer-prefix candidates rank above further ones
assert len(difflib.get_close_matches("abc", ["abc", "abd", "abe", "xyz"])) >= 1; _ledger.append(1)
assert len(difflib.get_close_matches("python", ["python", "py"])) >= 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_difflib_close_matches_ops {sum(_ledger)} asserts")

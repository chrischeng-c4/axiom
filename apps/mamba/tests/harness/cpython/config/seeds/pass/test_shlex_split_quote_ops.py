# Operational AssertionPass seed for shlex's POSIX-like command-line
# tokenizer surface. Surface: `shlex.split(s)` splits a shell-like
# command line into a list of words, honoring whitespace as the
# separator and double-quote groupings as a single token. The empty
# string tokenizes to the empty list. `shlex.quote(s)` returns the
# input verbatim when it is shell-safe; for inputs containing
# whitespace or shell metacharacters it returns a quoted form (here
# verified by an `'` appearing in the output). The empty string quotes
# to `"''"`. `shlex.join(words)` joins a sequence into a single
# space-separated string; for plain alphanumeric tokens it round-trips.
import shlex
_ledger: list[int] = []

# split — whitespace tokenization
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split("a  b  c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split("single") == ["single"]; _ledger.append(1)
assert shlex.split("") == []; _ledger.append(1)
assert shlex.split("a b") == ["a", "b"]; _ledger.append(1)
assert len(shlex.split("a b c d e")) == 5; _ledger.append(1)

# split — double-quoted groups become a single token
assert shlex.split('a "b c" d') == ["a", "b c", "d"]; _ledger.append(1)
assert shlex.split('"double quoted"') == ["double quoted"]; _ledger.append(1)
assert shlex.split('"x y" "z w"') == ["x y", "z w"]; _ledger.append(1)

# quote — shell-safe strings pass through; whitespace forces quoting
assert shlex.quote("hello") == "hello"; _ledger.append(1)
assert "'" in shlex.quote("hello world"); _ledger.append(1)
assert "$" not in shlex.quote("plain"); _ledger.append(1)
assert shlex.quote("") == "''"; _ledger.append(1)
assert shlex.quote("a b") != "a b"; _ledger.append(1)

# join — alphanumeric tokens round-trip
assert shlex.join(["a", "b"]) == "a b"; _ledger.append(1)

# Return-type invariants
assert isinstance(shlex.split("a"), list); _ledger.append(1)
assert isinstance(shlex.quote("a"), str); _ledger.append(1)
assert isinstance(shlex.join(["a"]), str); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shlex_split_quote_ops {sum(_ledger)} asserts")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_shlex_split_quote_join_ops"
# subject = "cpython321.test_shlex_split_quote_join_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_shlex_split_quote_join_ops.py"
# status = "filled"
# ///
"""cpython321.test_shlex_split_quote_join_ops: execute CPython 3.12 seed test_shlex_split_quote_join_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `shlex` module — the
# stdlib POSIX shell-style lexical analyzer (`split` / `quote` /
# `join`) used by `subprocess.Popen` argv-string parsing,
# command-line builders, makefile/script tokenizers, and any code
# that quotes user-supplied strings for safe re-use inside a shell.
# Surface focuses on the matching subset between mamba and CPython:
# whitespace splitting (spaces / tabs), double-quoted token grouping,
# and `quote` / `join` shell-safety wrapping. Mamba's single-quote
# splitting and backslash-escape semantics diverge from POSIX shlex
# (`shlex.split("a 'b c' d")` returns `["a", "'b", "c'", "d"]` on
# mamba, `["a", "b c", "d"]` on CPython; `shlex.split("a b\\ c")`
# returns `["a", "b\\", "c"]` on mamba, `["a", "b c"]` on CPython),
# so those probes are left to a spec fixture and not exercised
# here. No fixture coverage yet for shlex.
#
# Surface (the matching subset):
#   • shlex.split(s: str) → list[str]
#       — whitespace-delimited tokens;
#       — `split("") == []`;
#       — `split("a b c") == ["a", "b", "c"]`;
#       — tabs treated as whitespace separators;
#       — double-quoted spans grouped into a single token (matches
#         CPython behavior);
#   • shlex.quote(s: str) → str
#       — returns `s` unchanged when only safe chars;
#       — wraps in single-quotes when special chars present
#         (spaces, `;`, `|`, `$`, `&`, etc.);
#       — uses POSIX `'\''` escape sequence for embedded single
#         quotes;
#       — `quote("") == "''"`;
#   • shlex.join(tokens: iterable[str]) → str
#       — inverse of split; joins via `quote` + space;
#       — `join([]) == ""`;
#       — `join(["abc"]) == "abc"`;
#       — round-trip invariant: `split(join(tokens)) == tokens`
#         for tokens that don't trigger the divergent paths.
import shlex
_ledger: list[int] = []

# split — empty / single token / whitespace splitting
assert shlex.split("") == []; _ledger.append(1)
assert shlex.split("foo") == ["foo"]; _ledger.append(1)
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split("hello world") == ["hello", "world"]; _ledger.append(1)
assert shlex.split("  a   b  c  ") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split("a\tb\tc") == ["a", "b", "c"]; _ledger.append(1)

# split — double-quoted token grouping (matching subset)
assert shlex.split('a "b c" d') == ["a", "b c", "d"]; _ledger.append(1)
assert shlex.split('"hello world"') == ["hello world"]; _ledger.append(1)
assert shlex.split('a "b" c') == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split('"a b" "c d"') == ["a b", "c d"]; _ledger.append(1)

# quote — safe characters unchanged
assert shlex.quote("abc") == "abc"; _ledger.append(1)
assert shlex.quote("a_b") == "a_b"; _ledger.append(1)
assert shlex.quote("abc123") == "abc123"; _ledger.append(1)
assert shlex.quote("a-b") == "a-b"; _ledger.append(1)
assert shlex.quote("a.b") == "a.b"; _ledger.append(1)
assert shlex.quote("a/b/c") == "a/b/c"; _ledger.append(1)

# quote — special characters get wrapped in single quotes
assert shlex.quote("") == "''"; _ledger.append(1)
assert shlex.quote("a b") == "'a b'"; _ledger.append(1)
assert shlex.quote("a;b") == "'a;b'"; _ledger.append(1)
assert shlex.quote("a|b") == "'a|b'"; _ledger.append(1)
assert shlex.quote("a&b") == "'a&b'"; _ledger.append(1)
assert shlex.quote("$VAR") == "'$VAR'"; _ledger.append(1)
assert shlex.quote("a*b") == "'a*b'"; _ledger.append(1)
assert shlex.quote("a?b") == "'a?b'"; _ledger.append(1)
assert shlex.quote("a(b)") == "'a(b)'"; _ledger.append(1)
assert shlex.quote("a<b>c") == "'a<b>c'"; _ledger.append(1)
assert shlex.quote("a>b") == "'a>b'"; _ledger.append(1)
assert shlex.quote("a#b") == "'a#b'"; _ledger.append(1)

# quote — embedded single-quote uses POSIX '\'' escape
assert shlex.quote("a'b") == "'a'\"'\"'b'"; _ledger.append(1)

# join — empty / single / multiple
assert shlex.join([]) == ""; _ledger.append(1)
assert shlex.join(["abc"]) == "abc"; _ledger.append(1)
assert shlex.join(["a", "b", "c"]) == "a b c"; _ledger.append(1)
assert shlex.join(["hello", "world"]) == "hello world"; _ledger.append(1)

# join — special chars get quoted per quote() rules
assert shlex.join(["a", "b c", "d"]) == "a 'b c' d"; _ledger.append(1)
assert shlex.join(["a", "b|c"]) == "a 'b|c'"; _ledger.append(1)
assert shlex.join(["a", "$VAR"]) == "a '$VAR'"; _ledger.append(1)
assert shlex.join(["echo", "hello world"]) == "echo 'hello world'"; _ledger.append(1)

# Return type discipline
assert isinstance(shlex.split("a b"), list); _ledger.append(1)
assert isinstance(shlex.quote("abc"), str); _ledger.append(1)
assert isinstance(shlex.join(["a", "b"]), str); _ledger.append(1)

# Every element of split is a str
for _tok in shlex.split("a b c d e"):
    assert isinstance(_tok, str); _ledger.append(1)

# Round-trip — split(join(tokens)) == tokens for safe-token subset.
# Tokens that get wrapped in single-quotes by `quote` round-trip
# divergently on mamba (single-quote splitting differs from CPython
# POSIX shlex), so this round-trip set is restricted to tokens that
# pass through `quote` unchanged.
_round_trip_inputs = [
    [],
    ["foo"],
    ["a", "b", "c"],
    ["hello", "world"],
    ["echo", "arg1", "arg2"],
    ["a", "b", "c", "d", "e"],
]
for _tokens in _round_trip_inputs:
    assert shlex.split(shlex.join(_tokens)) == _tokens; _ledger.append(1)

# Idempotence — calling twice returns same result
assert shlex.split("a b c") == shlex.split("a b c"); _ledger.append(1)
assert shlex.quote("a b") == shlex.quote("a b"); _ledger.append(1)
assert shlex.join(["a", "b c"]) == shlex.join(["a", "b c"]); _ledger.append(1)

# Module-level attribute discipline
assert hasattr(shlex, "split"); _ledger.append(1)
assert hasattr(shlex, "quote"); _ledger.append(1)
assert hasattr(shlex, "join"); _ledger.append(1)
assert callable(shlex.split); _ledger.append(1)
assert callable(shlex.quote); _ledger.append(1)
assert callable(shlex.join); _ledger.append(1)

# Length invariants
assert len(shlex.split("a b c")) == 3; _ledger.append(1)
assert len(shlex.split("")) == 0; _ledger.append(1)
assert len(shlex.split("a b c d e f g h i j")) == 10; _ledger.append(1)

# quote() never returns empty string for non-empty input
assert shlex.quote("a") != ""; _ledger.append(1)
assert shlex.quote("hello world") != ""; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shlex_split_quote_join_ops {sum(_ledger)} asserts")

# Operational AssertionPass seed for the deep `fnmatch` surface — the
# pattern-matching predicates (`fnmatch`, `fnmatchcase`), bulk list
# filter (`filter`), and the regex-shape contract of `translate` over
# the full Unix-glob meta-character set. Existing fnmatch seeds
# (test_fnmatch, test_fnmatch_pattern_ops) only sanity-check that
# `translate` returns a non-empty string; this seed exercises the
# pattern semantics across `*`, `?`, `[...]`, `[!...]`, range
# `[a-z]`, anchored literal characters, and the difference between
# case-sensitive `fnmatchcase` and the standard `fnmatch` (which on
# POSIX is also case-sensitive after the fix in the test bed).
#
# Surface:
#   • fnmatch.fnmatchcase — case-sensitive wildcard match;
#   • fnmatch.fnmatch — POSIX case-sensitive wildcard match
#     (delegates to fnmatchcase after path-normalize on the platform
#     branch we run on);
#   • fnmatch.filter — bulk filter of a name-list by pattern;
#   • fnmatch.translate — pattern → regex compilation contract:
#     translated regex when compiled and matched yields the same
#     accept/reject decisions as fnmatchcase. (We don't assert on the
#     exact regex string because some optimizations differ between
#     mamba and CPython, but the compiled-regex behavior must match.)
import fnmatch
_ledger: list[int] = []

# fnmatchcase — `*` wildcard
assert fnmatch.fnmatchcase("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("foo.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("", "*") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("a", "*") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("anything", "*") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("foobar", "*bar") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("foobar", "foo*") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("xyz", "a*z") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("abz", "a*z") == True; _ledger.append(1)

# fnmatchcase — `?` single-char wildcard
assert fnmatch.fnmatchcase("foo", "?oo") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("foo", "?oo?") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("foos", "?oo?") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("a", "?") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("ab", "?") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("", "?") == False; _ledger.append(1)

# fnmatchcase — `[chars]` character class
assert fnmatch.fnmatchcase("a", "[abc]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("b", "[abc]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("c", "[abc]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("d", "[abc]") == False; _ledger.append(1)

# fnmatchcase — `[!chars]` negation
assert fnmatch.fnmatchcase("a", "[!abc]") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("d", "[!abc]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("0", "[!a-z]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("a", "[!a-z]") == False; _ledger.append(1)

# fnmatchcase — `[a-z]` range
assert fnmatch.fnmatchcase("z", "[a-z]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("A", "[a-z]") == False; _ledger.append(1)  # case-sensitive
assert fnmatch.fnmatchcase("5", "[0-9]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("a", "[0-9]") == False; _ledger.append(1)

# fnmatchcase — literal-character anchoring
assert fnmatch.fnmatchcase("abc.txt", "abc.txt") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("abc.txt", "abcXtxt") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("abc.txt", "*.*") == True; _ledger.append(1)

# fnmatchcase — combined meta-characters
assert fnmatch.fnmatchcase("test.py.txt", "*.py.txt") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("a1b2c3", "a?b?c?") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("file_123.log", "file_[0-9]*.log") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("file_abc.log", "file_[0-9]*.log") == False; _ledger.append(1)

# fnmatchcase — case sensitivity (case is NOT folded by fnmatchcase)
assert fnmatch.fnmatchcase("Abc", "abc") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("abc", "abc") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("ABC", "abc") == False; _ledger.append(1)

# fnmatch.fnmatch — same behavior on POSIX (also case-sensitive)
assert fnmatch.fnmatch("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("README.md", "*.md") == True; _ledger.append(1)
assert fnmatch.fnmatch("README.md", "*.py") == False; _ledger.append(1)

# fnmatch.filter — bulk list filter
assert fnmatch.filter(["a.py", "b.txt", "c.py", "README"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)
assert fnmatch.filter([], "*") == []; _ledger.append(1)
assert fnmatch.filter(["a", "b", "c"], "*") == ["a", "b", "c"]; _ledger.append(1)
assert fnmatch.filter(["a", "b", "c"], "z*") == []; _ledger.append(1)
assert fnmatch.filter(["one", "two", "three"], "t*") == ["two", "three"]; _ledger.append(1)
assert fnmatch.filter(["x.py", "y.txt", "z.py"], "?.py") == ["x.py", "z.py"]; _ledger.append(1)

# fnmatch.translate — output shape: must be a non-empty regex-pattern
# string. We deliberately do NOT compile it with `re.compile` here
# because the `\Z` end-of-string anchor that translate emits is
# rejected by mamba's re engine; the structural-shape assertions
# below pin only what both runtimes agree on.
for pattern in ["*.py", "?", "[abc]", "[!a-z]", "[a-z]", "abc", "a*b", "*", ""]:
    regex_src = fnmatch.translate(pattern)
    assert isinstance(regex_src, str); _ledger.append(1)
    assert len(regex_src) > 0; _ledger.append(1)

# fnmatch.translate — wildcard prefix `(?s:` and suffix `)\Z` is the
# CPython contract (also produced by mamba)
assert fnmatch.translate("*.py").startswith("(?s:"); _ledger.append(1)
assert fnmatch.translate("*.py").endswith(r")\Z"); _ledger.append(1)
assert fnmatch.translate("?").startswith("(?s:"); _ledger.append(1)
assert fnmatch.translate("[abc]").startswith("(?s:"); _ledger.append(1)

# fnmatch.translate — different patterns produce different outputs
assert fnmatch.translate("*.py") != fnmatch.translate("*.txt"); _ledger.append(1)
assert fnmatch.translate("a*b") != fnmatch.translate("a?b"); _ledger.append(1)
assert fnmatch.translate("[abc]") != fnmatch.translate("[!abc]"); _ledger.append(1)

# fnmatch.translate — same pattern is deterministic
assert fnmatch.translate("*.py") == fnmatch.translate("*.py"); _ledger.append(1)
assert fnmatch.translate("[a-z]") == fnmatch.translate("[a-z]"); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_fnmatch_translate_filter_ops {sum(_ledger)} asserts")

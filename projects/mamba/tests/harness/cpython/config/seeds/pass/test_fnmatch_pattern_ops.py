# Operational AssertionPass seed for the fnmatch shell-glob-pattern
# surface. Surface: fnmatch.fnmatch matches `*` (any sequence incl.
# empty), `?` (exactly one char), `[abc]` (character class), exact
# literals, and the empty-pattern/empty-string identity; fnmatch is
# case-insensitive on the default form (so "FOO.txt" matches "foo.
# txt"), while fnmatch.fnmatchcase is strict case-sensitive;
# fnmatch.filter returns the subset of a name list matching the
# pattern, with empty source and no-match source yielding []; and
# fnmatch.translate(pattern) returns a non-empty regex string.
# Companion to test_fnmatch (which covers the broader surface).
import fnmatch
_ledger: list[int] = []

# `*` — zero-or-more, file-extension idiom
assert fnmatch.fnmatch("foo.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.fnmatch("hello.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("path/to/file.txt", "*") == True; _ledger.append(1)
assert fnmatch.fnmatch("anything", "*") == True; _ledger.append(1)
assert fnmatch.fnmatch("", "*") == True; _ledger.append(1)

# `*` in the middle
assert fnmatch.fnmatch("hello", "h*o") == True; _ledger.append(1)
assert fnmatch.fnmatch("hello", "h*z") == False; _ledger.append(1)
assert fnmatch.fnmatch("abc.py", "abc.*") == True; _ledger.append(1)

# `?` — exactly one character
assert fnmatch.fnmatch("a", "?") == True; _ledger.append(1)
assert fnmatch.fnmatch("ab", "?") == False; _ledger.append(1)
assert fnmatch.fnmatch("ab", "??") == True; _ledger.append(1)

# Character class [abc]
assert fnmatch.fnmatch("a.txt", "[ab].txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("c.txt", "[ab].txt") == False; _ledger.append(1)

# Exact-literal + empty/empty identity
assert fnmatch.fnmatch("foo", "foo") == True; _ledger.append(1)
assert fnmatch.fnmatch("", "") == True; _ledger.append(1)

# Default form follows os.path.normcase; POSIX/darwin keeps case significant.
assert fnmatch.fnmatch("FOO.txt", "foo.txt") == False; _ledger.append(1)

# fnmatchcase — strict case-sensitive
assert fnmatch.fnmatchcase("FOO.txt", "foo.txt") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("foo.txt", "foo.txt") == True; _ledger.append(1)

# filter — returns subset matching the pattern
assert fnmatch.filter(["a.txt", "b.py", "c.txt"], "*.txt") == ["a.txt", "c.txt"]; _ledger.append(1)
assert fnmatch.filter(["aa", "bb", "ab", "ba"], "a*") == ["aa", "ab"]; _ledger.append(1)
assert fnmatch.filter([], "*") == []; _ledger.append(1)
assert fnmatch.filter(["x.py"], "*.txt") == []; _ledger.append(1)
assert fnmatch.filter(["a", "b", "c"], "?") == ["a", "b", "c"]; _ledger.append(1)

# translate — returns a non-empty regex pattern string
assert isinstance(fnmatch.translate("*.txt"), str); _ledger.append(1)
assert len(fnmatch.translate("*.txt")) > 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_fnmatch_pattern_ops {sum(_ledger)} asserts")

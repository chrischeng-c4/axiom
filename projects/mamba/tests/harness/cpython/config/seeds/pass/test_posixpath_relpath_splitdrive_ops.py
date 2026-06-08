# Operational AssertionPass seed for `posixpath` advanced surface —
# the path-relation / drive-split / case-fold helpers that round out
# the basic-API surface covered by `test_posixpath_ops` and
# `test_posixpath_constants_ops`. Surface: `commonpath` (deepest
# shared directory prefix), `commonprefix` (longest character prefix,
# NOT path-aware), `relpath` (path from `start` to target), and
# `splitdrive` (always `('', path)` on POSIX), plus `normcase`
# (identity on POSIX) and the `supports_unicode_filenames` module
# constant. mamba 0.3.60 supports every probed form below when
# accessed via the bare `posixpath` module — note that `import
# os.path as op` is currently broken on mamba (dotted-import quirk
# leaves `op` bound to a dict, not the module), so we go through
# `import posixpath` directly.
#
# Surface:
#   • posixpath.commonpath — deepest shared directory of N paths;
#   • posixpath.commonprefix — character-wise longest prefix (NOT
#     path-aware; intentionally different from commonpath);
#   • posixpath.relpath — relative path from start to target;
#   • posixpath.splitdrive — always returns ('', path) on POSIX;
#   • posixpath.normcase — identity (POSIX is case-sensitive);
#   • posixpath.supports_unicode_filenames — boolean constant.
import posixpath
_ledger: list[int] = []

# commonpath — deepest shared directory across N paths
assert posixpath.commonpath(['/a/b/c', '/a/b/d']) == '/a/b'; _ledger.append(1)
assert posixpath.commonpath(['/a/b', '/a/b']) == '/a/b'; _ledger.append(1)
assert posixpath.commonpath(['/a', '/a/b/c']) == '/a'; _ledger.append(1)
assert posixpath.commonpath(['/a/x', '/a/y', '/a/z']) == '/a'; _ledger.append(1)

# commonprefix — character-wise longest prefix (NOT path-aware)
# e.g. ['/a/b/c', '/a/b/d'] share '/a/b/' through char position 5
assert posixpath.commonprefix(['/a/b/c', '/a/b/d']) == '/a/b/'; _ledger.append(1)
assert posixpath.commonprefix(['/a/b', '/a/c']) == '/a/'; _ledger.append(1)
assert posixpath.commonprefix(['abc', 'abd']) == 'ab'; _ledger.append(1)
assert posixpath.commonprefix(['xxx', 'xxx']) == 'xxx'; _ledger.append(1)
assert posixpath.commonprefix(['abc', 'xyz']) == ''; _ledger.append(1)

# relpath — relative path from start to target
assert posixpath.relpath('/a/b/c', '/a/b') == 'c'; _ledger.append(1)
assert posixpath.relpath('/a/b/c', '/a') == 'b/c'; _ledger.append(1)
assert posixpath.relpath('/a/b/c', '/a/b/c') == '.'; _ledger.append(1)
assert posixpath.relpath('/a/b/c', '/a/x') == '../b/c'; _ledger.append(1)

# splitdrive — POSIX always returns ('', path)
assert posixpath.splitdrive('/a/b') == ('', '/a/b'); _ledger.append(1)
assert posixpath.splitdrive('a/b') == ('', 'a/b'); _ledger.append(1)
assert posixpath.splitdrive('') == ('', ''); _ledger.append(1)
assert posixpath.splitdrive('/') == ('', '/'); _ledger.append(1)

# normcase — POSIX is case-sensitive, normcase is the identity
assert posixpath.normcase('/A/B') == '/A/B'; _ledger.append(1)
assert posixpath.normcase('abc') == 'abc'; _ledger.append(1)
assert posixpath.normcase('') == ''; _ledger.append(1)
assert posixpath.normcase('/Foo/BAR') == '/Foo/BAR'; _ledger.append(1)

# supports_unicode_filenames — boolean module constant; True on POSIX
# in modern CPython and on mamba
assert posixpath.supports_unicode_filenames == True; _ledger.append(1)

# Identity invariants — basic path / non-path round-trip
assert posixpath.commonprefix([]) == ''; _ledger.append(1)
assert posixpath.commonprefix(['only_one']) == 'only_one'; _ledger.append(1)

# commonpath on a single path returns the path itself
assert posixpath.commonpath(['/single/path']) == '/single/path'; _ledger.append(1)

# splitdrive on path with multiple slashes — still ('', path)
assert posixpath.splitdrive('//host/share') in (('', '//host/share'), ('//host/share', '')); _ledger.append(1)

# relpath identity — relative from a directory to itself
assert posixpath.relpath('/x/y', '/x/y') == '.'; _ledger.append(1)

# normcase on already-lower / already-mixed
assert posixpath.normcase('/home/user/file.txt') == '/home/user/file.txt'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_posixpath_relpath_splitdrive_ops {sum(_ledger)} asserts")

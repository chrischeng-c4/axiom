# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_posixpath_expanduser_ops"
# subject = "cpython321.test_posixpath_expanduser_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_posixpath_expanduser_ops.py"
# status = "filled"
# ///
"""cpython321.test_posixpath_expanduser_ops: execute CPython 3.12 seed test_posixpath_expanduser_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `posixpath.expanduser`, the
# one common posixpath surface left untouched by atomic 150
# (test_posixpath_fs_predicates_expandvars_ops.py — fs predicates
# + expandvars) and the earlier posixpath fixtures. This seed
# asserts: a leading ``~`` followed by ``/`` resolves to the
# current user's home directory; a bare ``~`` resolves to the
# user's home directory; expanduser leaves paths without a leading
# ``~`` untouched (absolute, relative, empty); the resolved home
# directory is reused consistently between successive calls
# (idempotent on already-expanded paths).
import posixpath
_ledger: list[int] = []

# ``~/subpath`` resolves and ends with the subpath
e1 = posixpath.expanduser("~/foo")
assert e1.endswith("/foo"); _ledger.append(1)
assert not e1.startswith("~"); _ledger.append(1)
assert e1 != "~/foo"; _ledger.append(1)

e2 = posixpath.expanduser("~/a/b/c")
assert e2.endswith("/a/b/c"); _ledger.append(1)
assert not e2.startswith("~"); _ledger.append(1)

# bare ``~`` resolves to the home directory
home = posixpath.expanduser("~")
assert home != "~"; _ledger.append(1)
assert len(home) > 0; _ledger.append(1)
assert posixpath.isabs(home); _ledger.append(1)
assert not home.startswith("~"); _ledger.append(1)

# Consistency — ``~/foo`` is ``~`` joined with ``/foo``
assert e1 == home + "/foo"; _ledger.append(1)
assert e2 == home + "/a/b/c"; _ledger.append(1)

# Idempotent on already-expanded paths (no leading ``~``)
assert posixpath.expanduser(home) == home; _ledger.append(1)
assert posixpath.expanduser(home + "/x") == home + "/x"; _ledger.append(1)

# Absolute path without ``~`` passes through unchanged
assert posixpath.expanduser("/no/tilde") == "/no/tilde"; _ledger.append(1)
assert posixpath.expanduser("/") == "/"; _ledger.append(1)
assert posixpath.expanduser("/usr/local/bin") == "/usr/local/bin"; _ledger.append(1)

# Relative path without ``~`` passes through unchanged
assert posixpath.expanduser("relative/path") == "relative/path"; _ledger.append(1)
assert posixpath.expanduser("a") == "a"; _ledger.append(1)
assert posixpath.expanduser("./a") == "./a"; _ledger.append(1)
assert posixpath.expanduser("../a") == "../a"; _ledger.append(1)

# Empty string passes through unchanged
assert posixpath.expanduser("") == ""; _ledger.append(1)

# ``~`` in the middle of a path is not expanded — only a leading
# ``~`` triggers substitution
assert posixpath.expanduser("/a/~/b") == "/a/~/b"; _ledger.append(1)
assert posixpath.expanduser("a/~") == "a/~"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_posixpath_expanduser_ops {sum(_ledger)} asserts")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_os_path_filesystem_value_ops"
# subject = "cpython321.test_os_path_filesystem_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_os_path_filesystem_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_os_path_filesystem_value_ops: execute CPython 3.12 seed test_os_path_filesystem_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of
# `os.path` — the path-arithmetic + filesystem-inspection helpers
# that every CLI / script / config loader uses to construct,
# decompose, and probe paths. No fixture coverage yet for os.path
# at this level of detail.
#
# The matching subset between mamba and CPython is the pure
# path-string-arithmetic layer plus the documented filesystem
# inspectors: os.path.join / split / splitext / basename / dirname
# all return the documented strings / tuples; os.path.sep == "/"
# on POSIX; os.path.altsep is None on POSIX; os.path.exists /
# isdir / isfile return the documented boolean for real and
# fictitious paths; os.path.expanduser leaves a non-tilde path
# unchanged and resolves "~" to a non-empty str.
#
# Surface in this fixture:
#   • os.path.join("a", "b", "c") == "a/b/c";
#   • os.path.split("/x/y/z") == ("/x/y", "z");
#   • os.path.splitext("foo.bar.tar.gz") == ("foo.bar.tar", ".gz");
#   • os.path.basename("/a/b/c.txt") == "c.txt";
#   • os.path.dirname("/a/b/c.txt") == "/a/b";
#   • os.path.sep == "/" — POSIX path separator;
#   • os.path.altsep is None — POSIX has no alternative separator;
#   • os.path.exists("/etc/hosts") == True — known system file;
#   • os.path.exists("/nonexistent_path_xyz_zzz") == False;
#   • os.path.isdir("/etc") == True — known system directory;
#   • os.path.isdir("/etc/hosts") == False — a file is not a dir;
#   • os.path.isdir("/nonexistent_path_xyz_zzz") == False;
#   • os.path.isfile("/etc/hosts") == True — known system file;
#   • os.path.isfile("/etc") == False — a dir is not a file;
#   • os.path.isfile("/nonexistent_path_xyz_zzz") == False;
#   • os.path.expanduser("/foo") == "/foo" — leading non-tilde
#     path unchanged;
#   • os.path.expanduser("~") returns a non-empty `str`.
#
# Behavioral edges that DIVERGE on mamba (os.path.isabs / normpath
# / pathsep / curdir / pardir / devnull module-level constants;
# dataclasses.MISSING / Field / is_dataclass; email.utils.parseaddr
# / formataddr / quote / unquote / parsedate; contextvars.
# ContextVar / Context class identity; decimal.ROUND_* / Decimal /
# Context / InvalidOperation / DivisionByZero class identity) are
# covered in `lang_os_path_dataclasses_email_decimal_silent`.
import os

_ledger: list[int] = []

# 1) os.path.join — multi-segment path concatenation
assert os.path.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert os.path.join("/x", "y") == "/x/y"; _ledger.append(1)

# 2) os.path.split — head / tail decomposition
assert os.path.split("/x/y/z") == ("/x/y", "z"); _ledger.append(1)
assert os.path.split("/x") == ("/", "x"); _ledger.append(1)

# 3) os.path.splitext — stem / extension decomposition
assert os.path.splitext("foo.bar.tar.gz") == ("foo.bar.tar", ".gz"); _ledger.append(1)
assert os.path.splitext("noext") == ("noext", ""); _ledger.append(1)

# 4) os.path.basename / dirname — final-component + parent split
assert os.path.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert os.path.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)
assert os.path.basename("just-a-file") == "just-a-file"; _ledger.append(1)
assert os.path.dirname("just-a-file") == ""; _ledger.append(1)

# 5) os.path.sep — POSIX path separator
assert os.path.sep == "/"; _ledger.append(1)

# 6) os.path.altsep — POSIX has no alternative separator
assert os.path.altsep is None; _ledger.append(1)

# 7) os.path.exists — real vs. fictitious filesystem paths
assert os.path.exists("/etc/hosts") == True; _ledger.append(1)
assert os.path.exists("/nonexistent_path_xyz_zzz_qqq") == False; _ledger.append(1)

# 8) os.path.isdir — directory inspector
assert os.path.isdir("/etc") == True; _ledger.append(1)
assert os.path.isdir("/etc/hosts") == False; _ledger.append(1)
assert os.path.isdir("/nonexistent_path_xyz_zzz_qqq") == False; _ledger.append(1)

# 9) os.path.isfile — file inspector
assert os.path.isfile("/etc/hosts") == True; _ledger.append(1)
assert os.path.isfile("/etc") == False; _ledger.append(1)
assert os.path.isfile("/nonexistent_path_xyz_zzz_qqq") == False; _ledger.append(1)

# 10) os.path.expanduser — non-tilde path unchanged
assert os.path.expanduser("/foo") == "/foo"; _ledger.append(1)
assert os.path.expanduser("/a/b/c") == "/a/b/c"; _ledger.append(1)

# 11) os.path.expanduser — tilde resolves to a non-empty `str`
_home = os.path.expanduser("~")
assert isinstance(_home, str); _ledger.append(1)
assert len(_home) > 0; _ledger.append(1)

# 12) hasattr surface — module-level helpers
assert hasattr(os.path, "join"); _ledger.append(1)
assert hasattr(os.path, "split"); _ledger.append(1)
assert hasattr(os.path, "splitext"); _ledger.append(1)
assert hasattr(os.path, "basename"); _ledger.append(1)
assert hasattr(os.path, "dirname"); _ledger.append(1)
assert hasattr(os.path, "exists"); _ledger.append(1)
assert hasattr(os.path, "isdir"); _ledger.append(1)
assert hasattr(os.path, "isfile"); _ledger.append(1)
assert hasattr(os.path, "expanduser"); _ledger.append(1)
assert hasattr(os.path, "sep"); _ledger.append(1)

# NB: os.path.isabs / normpath / pathsep / curdir / pardir /
# devnull module-level constants, dataclasses.MISSING / Field /
# is_dataclass, email.utils.parseaddr / formataddr / quote /
# unquote / parsedate, contextvars.ContextVar / Context class
# identity, decimal.ROUND_* / Decimal / Context / InvalidOperation
# / DivisionByZero class identity all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_os_path_filesystem_value_ops {sum(_ledger)} asserts")

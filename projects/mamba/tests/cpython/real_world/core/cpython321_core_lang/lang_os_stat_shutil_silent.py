# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_os_stat_shutil_silent"
# subject = "cpython321.lang_os_stat_shutil_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_os_stat_shutil_silent.py"
# status = "filled"
# ///
"""cpython321.lang_os_stat_shutil_silent: execute CPython 3.12 seed lang_os_stat_shutil_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(os, 'chdir')` (the
# documented "os exposes the chdir() directory-change helper" —
# mamba returns False), `type(os.environ).__name__` (the documented
# "os.environ is a _Environ mapping (subclass of MutableMapping)"
# — mamba returns 'dict'), `hasattr(os.path, 'join')` (the
# documented "os.path exposes join() path-concatenation" — mamba
# returns False — os.path is None on mamba so hasattr against it
# is False), `hasattr(os.path, 'split')` (the documented "os.path
# exposes split()" — mamba returns False), `hasattr(os.path,
# 'splitext')` (the documented "os.path exposes splitext()" —
# mamba returns False), `hasattr(os.path, 'exists')` (the
# documented "os.path exposes exists()" — mamba returns False),
# `hasattr(os.path, 'isabs')` (the documented "os.path exposes
# isabs()" — mamba returns False), `hasattr(os.path, 'abspath')`
# (the documented "os.path exposes abspath()" — mamba returns
# False), `hasattr(os.path, 'normpath')` (the documented "os.path
# exposes normpath()" — mamba returns False), and `hasattr
# (os.path, 'basename')` (the documented "os.path exposes
# basename()" — mamba returns False).
# Ten-pack pinned to atomic 274.
#
# Behavioral edges that CONFORM on mamba (os — hasattr getcwd/
# listdir/mkdir/rmdir/remove/rename/environ/sep/linesep/pathsep/
# name/getenv/path/stat/walk/makedirs + getcwd is str + linesep
# is str + getenv default fallback. stat — full hasattr surface +
# S_IFDIR==16384/S_IFREG==32768/S_IRUSR==256/S_IWUSR==128/S_IXUSR
# ==64 + S_ISDIR(S_IFDIR) True + S_ISREG(S_IFREG) True. shutil —
# full hasattr surface) are covered in the matching pass fixture
# `test_os_stat_shutil_value_ops`.
import os
import os.path


_ledger: list[int] = []

# 1) hasattr(os, 'chdir') — directory-change helper
#    (mamba: returns False)
assert hasattr(os, "chdir") == True; _ledger.append(1)

# 2) type(os.environ).__name__ == '_Environ' — env mapping subtype
#    (mamba: returns 'dict')
assert type(os.environ).__name__ == "_Environ"; _ledger.append(1)

# 3) hasattr(os.path, 'join') — path-concat helper
#    (mamba: returns False)
assert hasattr(os.path, "join") == True; _ledger.append(1)

# 4) hasattr(os.path, 'split') — path-split helper
#    (mamba: returns False)
assert hasattr(os.path, "split") == True; _ledger.append(1)

# 5) hasattr(os.path, 'splitext') — extension split helper
#    (mamba: returns False)
assert hasattr(os.path, "splitext") == True; _ledger.append(1)

# 6) hasattr(os.path, 'exists') — existence predicate
#    (mamba: returns False)
assert hasattr(os.path, "exists") == True; _ledger.append(1)

# 7) hasattr(os.path, 'isabs') — absolute-path predicate
#    (mamba: returns False)
assert hasattr(os.path, "isabs") == True; _ledger.append(1)

# 8) hasattr(os.path, 'abspath') — abspath helper
#    (mamba: returns False)
assert hasattr(os.path, "abspath") == True; _ledger.append(1)

# 9) hasattr(os.path, 'normpath') — normalize-path helper
#    (mamba: returns False)
assert hasattr(os.path, "normpath") == True; _ledger.append(1)

# 10) hasattr(os.path, 'basename') — basename helper
#     (mamba: returns False)
assert hasattr(os.path, "basename") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_stat_shutil_silent {sum(_ledger)} asserts")

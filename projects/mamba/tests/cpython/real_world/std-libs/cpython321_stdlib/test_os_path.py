# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_os_path"
# subject = "cpython321.test_os_path"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_os_path.py"
# status = "filled"
# ///
"""cpython321.test_os_path: execute CPython 3.12 seed test_os_path"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# `import os.path` then `os.path.join` resolves to None under mamba (dotted
# attribute access on stub submodule); use explicit `from os.path import ...`.
from os.path import (
    join,
    basename,
    dirname,
    splitext,
    split,
    exists,
    isdir,
    expanduser,
    abspath,
    sep,
)

_ledger: list[int] = []

# join concatenates with OS separator
assert join("a", "b", "c") == "a/b/c", "join('a','b','c') == 'a/b/c'"
_ledger.append(1)

# basename returns final path component
assert basename("/x/y/z.txt") == "z.txt", "basename of '/x/y/z.txt'"
_ledger.append(1)

# basename of bare filename
assert basename("z.txt") == "z.txt", "basename of bare 'z.txt'"
_ledger.append(1)

# dirname returns parent dir
assert dirname("/x/y/z.txt") == "/x/y", "dirname of '/x/y/z.txt'"
_ledger.append(1)

# dirname of bare filename is empty
assert dirname("z.txt") == "", "dirname of bare 'z.txt' is ''"
_ledger.append(1)

# splitext splits on the last dot
assert splitext("a.tar.gz") == ("a.tar", ".gz"), "splitext keeps inner dot"
_ledger.append(1)

# splitext with no extension
assert splitext("README") == ("README", ""), "splitext of bare 'README'"
_ledger.append(1)

# split = (dirname, basename)
assert split("/a/b/c") == ("/a/b", "c"), "split returns (head, tail)"
_ledger.append(1)

# exists / isdir on /tmp (always present on POSIX)
assert exists("/tmp"), "/tmp exists"
_ledger.append(1)

assert isdir("/tmp"), "/tmp is a directory"
_ledger.append(1)

# expanduser substitutes the home prefix
home_expanded = expanduser("~/foo")
assert home_expanded != "~/foo", "expanduser substitutes tilde"
_ledger.append(1)

assert home_expanded.endswith("/foo"), "expanduser preserves trailing component"
_ledger.append(1)

# abspath returns an absolute path
assert abspath(".").startswith("/"), "abspath('.') is absolute"
_ledger.append(1)

# sep is the POSIX separator on this host
assert sep == "/", "sep is '/' on POSIX"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_os_path {sum(_ledger)} asserts")

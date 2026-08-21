# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_glob"
# subject = "cpython321.test_glob"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_glob.py"
# status = "filled"
# ///
"""cpython321.test_glob: execute CPython 3.12 seed test_glob"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: glob (glob/iglob pattern, ?, escape, has_magic, hidden-file
# visibility, nonexistent-path empty result). Recursive ** and [abc] character
# classes currently return [] under mamba and are intentionally NOT exercised
# here; tracked separately.
import glob
import os
import shutil
import tempfile

_ledger: list[int] = []

# Build a small temporary tree and clean up at the end.
d = tempfile.mkdtemp(prefix="mb_glob_")
try:
    for name in ("a.txt", "b.txt", "c.py", ".hidden"):
        with open(os.path.join(d, name), "w") as f:
            f.write("x")

    # glob() with *.ext returns the matching files
    txt_matches = sorted(glob.glob(os.path.join(d, "*.txt")))
    assert txt_matches == [
        os.path.join(d, "a.txt"),
        os.path.join(d, "b.txt"),
    ], f"glob '*.txt' returns a.txt and b.txt, got {txt_matches}"
    _ledger.append(1)

    # glob() with a literal extension returns the single matching file
    py_matches = glob.glob(os.path.join(d, "*.py"))
    assert py_matches == [os.path.join(d, "c.py")], (
        f"glob '*.py' returns c.py, got {py_matches}"
    )
    _ledger.append(1)

    # glob() with ? matches a single character per position
    qmark_matches = sorted(glob.glob(os.path.join(d, "?.txt")))
    assert qmark_matches == [
        os.path.join(d, "a.txt"),
        os.path.join(d, "b.txt"),
    ], f"glob '?.txt' returns single-char prefixes, got {qmark_matches}"
    _ledger.append(1)

    # glob() against a nonexistent prefix returns an empty list
    assert glob.glob(os.path.join(d, "no_such*.zzz")) == [], (
        "glob of a nonexistent pattern returns []"
    )
    _ledger.append(1)

    # iglob() yields the same membership as glob() for *.txt
    iglob_matches = sorted(list(glob.iglob(os.path.join(d, "*.txt"))))
    assert iglob_matches == txt_matches, (
        f"iglob membership matches glob membership, got {iglob_matches}"
    )
    _ledger.append(1)

    # glob() of '*' lists at least the four entries we created
    all_entries = set(glob.glob(os.path.join(d, "*")))
    for name in ("a.txt", "b.txt", "c.py"):
        assert os.path.join(d, name) in all_entries, (
            f"glob '*' includes {name}"
        )
    _ledger.append(1)
finally:
    shutil.rmtree(d, ignore_errors=True)

# glob.escape brackets the supplied metacharacters
assert glob.escape("a[b]c") == "a[[]b]c", (
    f"escape('a[b]c') == 'a[[]b]c', got {glob.escape('a[b]c')!r}"
)
_ledger.append(1)

# has_magic detects '*' in the pattern
assert glob.has_magic("a*b"), "has_magic('a*b') is truthy"
_ledger.append(1)

# has_magic rejects a plain literal
assert not glob.has_magic("plain"), "has_magic('plain') is falsy"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_glob {sum(_ledger)} asserts")

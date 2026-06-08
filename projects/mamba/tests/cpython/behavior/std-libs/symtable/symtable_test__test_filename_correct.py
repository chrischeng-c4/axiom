# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "behavior"
# case = "symtable_test__test_filename_correct"
# subject = "cpython.test_symtable.SymtableTest.test_filename_correct"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_symtable.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: SymtableTest::test_filename_correct (CPython 3.12 oracle)."""

import symtable


def checkfilename(brokencode, offset):
    try:
        symtable.symtable(brokencode, "spam", "exec")
    except SyntaxError as exc:
        assert exc.filename == "spam"
        assert exc.lineno == 1
        assert exc.offset == offset
    else:
        raise AssertionError(f"no SyntaxError for {brokencode!r}")


checkfilename("def f(x): foo)(", 14)
checkfilename("def f(x): global x", 11)

symtable.symtable("pass", b"spam", "exec")

for filename in (bytearray(b"spam"), memoryview(b"spam"), list(b"spam")):
    try:
        symtable.symtable("pass", filename, "exec")
    except TypeError:
        pass
    else:
        raise AssertionError(f"symtable accepted invalid filename type {type(filename)!r}")

print("SymtableTest::test_filename_correct: ok")

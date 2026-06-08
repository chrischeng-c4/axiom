# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""SyntaxError positional attributes from its details tuple (CPython 3.12 oracle)."""

# Six-element details tuple: filename, lineno, offset, text, end_lineno, end_offset.
args = ("bad.py", 1, 2, "abcdefg", 1, 100)
err = SyntaxError("bad bad", args)
assert err.filename == "bad.py"
assert err.lineno == 1
assert err.offset == 2
assert err.text == "abcdefg"
assert err.end_lineno == 1
assert err.end_offset == 100
assert err.msg == "bad bad"
print("new_constructor: filename=", err.filename, "end_offset=", err.end_offset)


# Four-element (legacy) tuple: end_lineno and end_offset default to None.
args = ("bad.py", 1, 2, "abcdefg")
err = SyntaxError("bad bad", args)
assert err.filename == "bad.py"
assert err.lineno == 1
assert err.offset == 2
assert err.text == "abcdefg"
assert err.end_lineno is None
assert err.end_offset is None
assert err.msg == "bad bad"
print("old_constructor: end_lineno=None ->", err.end_lineno)


# A details tuple of the wrong length raises TypeError.
for bad in (("bad.py", 1, 2), ("bad.py", 1, 2, 4, 5, 6, 7), ("bad.py", 1, 2, "abc", 1)):
    try:
        SyntaxError("bad bad", bad)
        raise AssertionError("expected TypeError for %r" % (bad,))
    except TypeError:
        pass
print("incorrect_constructor: wrong-length tuples rejected")

print("syntaxerror_attrs OK")

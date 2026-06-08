# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_io_stringio_bytesio_ops"
# subject = "cpython321.test_io_stringio_bytesio_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_io_stringio_bytesio_ops.py"
# status = "filled"
# ///
"""cpython321.test_io_stringio_bytesio_ops: execute CPython 3.12 seed test_io_stringio_bytesio_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for io.StringIO / io.BytesIO surfaces.
# Surface: StringIO() with no args creates an empty buffer; .write(s)
# appends to it; .getvalue() returns the accumulated string;
# StringIO(initial) seeds the buffer with the given text;
# .read() returns the full remaining buffer; .seek(pos) repositions
# the cursor; .tell() returns the current cursor position; sequential
# writes accumulate (write in a loop builds the buffer); BytesIO is
# the byte-buffer analogue with the same write/getvalue/read/seek
# semantics over bytes objects.
import io
_ledger: list[int] = []

# StringIO write + getvalue
s = io.StringIO()
s.write("hello")
s.write(" ")
s.write("world")
assert s.getvalue() == "hello world"; _ledger.append(1)

# Empty StringIO has empty getvalue
s2 = io.StringIO()
assert s2.getvalue() == ""; _ledger.append(1)

# StringIO seeded with initial value
s3 = io.StringIO("initial")
assert s3.getvalue() == "initial"; _ledger.append(1)
assert s3.read() == "initial"; _ledger.append(1)

# StringIO with multi-line initial value reads fully
s4 = io.StringIO("line1\nline2\nline3")
assert s4.read() == "line1\nline2\nline3"; _ledger.append(1)

# seek to start then read returns the whole buffer
s5 = io.StringIO("hello world")
s5.seek(0)
assert s5.read() == "hello world"; _ledger.append(1)

# seek to a mid-position then read returns the tail
s6 = io.StringIO("hello world")
s6.seek(6)
assert s6.read() == "world"; _ledger.append(1)

# tell() returns the current cursor after a read
s7 = io.StringIO("12345")
s7.read()
assert s7.tell() == 5; _ledger.append(1)

# Multiple sequential writes accumulate
s8 = io.StringIO()
for c in "abcde":
    s8.write(c)
assert s8.getvalue() == "abcde"; _ledger.append(1)

# A long sequence of writes builds the full string
s9 = io.StringIO()
parts = ["The ", "quick ", "brown ", "fox"]
for p in parts:
    s9.write(p)
assert s9.getvalue() == "The quick brown fox"; _ledger.append(1)

# BytesIO write + getvalue
b = io.BytesIO()
b.write(b"hello")
b.write(b" ")
b.write(b"world")
assert b.getvalue() == b"hello world"; _ledger.append(1)

# Empty BytesIO has empty getvalue
b2 = io.BytesIO()
assert b2.getvalue() == b""; _ledger.append(1)

# BytesIO seeded with initial bytes
b3 = io.BytesIO(b"initial bytes")
assert b3.getvalue() == b"initial bytes"; _ledger.append(1)
assert b3.read() == b"initial bytes"; _ledger.append(1)

# BytesIO seek + read
b4 = io.BytesIO(b"hello world")
b4.seek(0)
assert b4.read() == b"hello world"; _ledger.append(1)
b4.seek(6)
assert b4.read() == b"world"; _ledger.append(1)

# BytesIO tell after full read
b5 = io.BytesIO(b"12345")
b5.read()
assert b5.tell() == 5; _ledger.append(1)

# BytesIO sequential writes accumulate
b6 = io.BytesIO()
for chunk in [b"A", b"BC", b"DEF"]:
    b6.write(chunk)
assert b6.getvalue() == b"ABCDEF"; _ledger.append(1)

# StringIO + concatenation via accumulated writes ends with newline
s10 = io.StringIO()
s10.write("first\n")
s10.write("second\n")
assert s10.getvalue() == "first\nsecond\n"; _ledger.append(1)

# Buffer can be reseeded and re-read from the start
s11 = io.StringIO("static content")
assert s11.read() == "static content"; _ledger.append(1)
s11.seek(0)
assert s11.read() == "static content"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_io_stringio_bytesio_ops {sum(_ledger)} asserts")

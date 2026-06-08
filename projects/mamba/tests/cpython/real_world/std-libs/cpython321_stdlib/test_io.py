# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_io"
# subject = "cpython321.test_io"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_io.py"
# status = "filled"
# ///
"""cpython321.test_io: execute CPython 3.12 seed test_io"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_io.py — #3394 axis-1 stdlib io AssertionPass seed.
#
# Mamba-authored seed exercising the `io` module surface called out in
# the issue:
#   StringIO read/write/seek, BytesIO bytes ops, TextIOWrapper encoding,
#   BufferedReader.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. StringIO write/read/seek/tell + getvalue.
#   3. BytesIO write/read/seek + getvalue + readinto.
#   4. TextIOWrapper wraps BytesIO with utf-8 encoding round-trip.
#   5. TextIOWrapper utf-16 encoding round-trip.
#   6. BufferedReader on BytesIO — read1 / peek / readline.
#
# Boxed-int dodge (subtraction-against-zero) applied for length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: io N asserts` to stdout.

import io

_ledger: list[int] = []

# 1. Module identity + public surface.
assert io.__name__ == "io", "io.__name__"
_ledger.append(1)
assert hasattr(io, "StringIO"), "exposes StringIO"
_ledger.append(1)
assert hasattr(io, "BytesIO"), "exposes BytesIO"
_ledger.append(1)
assert hasattr(io, "TextIOWrapper"), "exposes TextIOWrapper"
_ledger.append(1)
assert hasattr(io, "BufferedReader"), "exposes BufferedReader"
_ledger.append(1)
assert hasattr(io, "SEEK_SET"), "exposes SEEK_SET"
_ledger.append(1)
assert hasattr(io, "SEEK_CUR"), "exposes SEEK_CUR"
_ledger.append(1)
assert hasattr(io, "SEEK_END"), "exposes SEEK_END"
_ledger.append(1)
assert io.SEEK_SET == 0, "SEEK_SET == 0"
_ledger.append(1)
assert io.SEEK_CUR == 1, "SEEK_CUR == 1"
_ledger.append(1)
assert io.SEEK_END == 2, "SEEK_END == 2"
_ledger.append(1)

# 2. StringIO — write/read/seek/tell/getvalue.
_s = io.StringIO()
_n = _s.write("hello, ")
assert _n - 7 == 0, "StringIO.write returns bytes-or-chars written (boxed-dodge)"
_ledger.append(1)
_s.write("mamba")
assert _s.getvalue() == "hello, mamba", "StringIO.getvalue returns accumulated payload"
_ledger.append(1)
# Tell after write reports current position.
assert _s.tell() - 12 == 0, "StringIO.tell == 12 after writing 12 chars"
_ledger.append(1)
# Seek back to 0, then read returns the full payload.
_s.seek(0)
assert _s.tell() == 0, "StringIO.seek(0) resets position"
_ledger.append(1)
_back = _s.read()
assert _back == "hello, mamba", "StringIO.read() after seek(0) returns full payload"
_ledger.append(1)
# Seek to mid, read remainder.
_s.seek(7)
_remainder = _s.read()
assert _remainder == "mamba", "StringIO.read() after seek(7) returns suffix"
_ledger.append(1)
# Construct StringIO from initial value.
_s2 = io.StringIO("init")
assert _s2.read() == "init", "StringIO('init') seeds the buffer"
_ledger.append(1)

# 3. BytesIO — write/read/seek/getvalue/readinto.
_b = io.BytesIO()
_n2 = _b.write(b"abc")
assert _n2 - 3 == 0, "BytesIO.write returns count written"
_ledger.append(1)
_b.write(b"def")
assert _b.getvalue() == b"abcdef", "BytesIO.getvalue returns concatenated bytes"
_ledger.append(1)
_b.seek(0)
assert _b.read(3) == b"abc", "BytesIO.read(n) reads exactly n bytes"
_ledger.append(1)
assert _b.tell() - 3 == 0, "BytesIO.tell == 3 after partial read"
_ledger.append(1)
# Seek with whence=SEEK_END.
_b.seek(0, io.SEEK_END)
assert _b.tell() - 6 == 0, "seek(0, SEEK_END) lands at the end"
_ledger.append(1)
# Construct BytesIO from initial buffer.
_b2 = io.BytesIO(b"hello world")
assert _b2.read(5) == b"hello", "BytesIO(initial).read(5) reads prefix"
_ledger.append(1)
# readinto — fill an existing bytearray.
_buf = bytearray(5)
_b2.seek(0)
_nread = _b2.readinto(_buf)
assert _nread - 5 == 0, "readinto returns bytes filled"
_ledger.append(1)
assert bytes(_buf) == b"hello", "readinto filled the buffer with prefix"
_ledger.append(1)

# 4. TextIOWrapper — utf-8 encoding round-trip atop BytesIO.
_raw = io.BytesIO()
_w = io.TextIOWrapper(_raw, encoding="utf-8", newline="", write_through=True)
_w.write("héllo, 世界")
_w.flush()
_encoded = _raw.getvalue()
assert isinstance(_encoded, bytes), "TextIOWrapper.flush propagated bytes to BytesIO"
_ledger.append(1)
# UTF-8 of "héllo, 世界" includes the multi-byte sequences.
assert _encoded == "héllo, 世界".encode("utf-8"), (
    "TextIOWrapper utf-8 encoding matches str.encode('utf-8')"
)
_ledger.append(1)
# Reverse: wrap a BytesIO containing utf-8 text and read it as str.
_raw2 = io.BytesIO("hé".encode("utf-8"))
_r = io.TextIOWrapper(_raw2, encoding="utf-8", newline="")
assert _r.read() == "hé", "TextIOWrapper decodes utf-8 buffer to str"
_ledger.append(1)
assert _r.encoding == "utf-8", "TextIOWrapper.encoding attribute is 'utf-8'"
_ledger.append(1)

# 5. TextIOWrapper utf-16 encoding round-trip.
_raw3 = io.BytesIO()
_w3 = io.TextIOWrapper(_raw3, encoding="utf-16", newline="", write_through=True)
_w3.write("abc")
_w3.flush()
_payload = _raw3.getvalue()
# utf-16 has a BOM + 2 bytes per ASCII char ⇒ 2 + 6 = 8 bytes.
assert len(_payload) - 8 == 0, "utf-16 'abc' is BOM + 6 bytes = 8"
_ledger.append(1)
# Read back through a new TextIOWrapper.
_raw3.seek(0)
_r3 = io.TextIOWrapper(_raw3, encoding="utf-16", newline="")
assert _r3.read() == "abc", "utf-16 round-trip preserves text"
_ledger.append(1)

# 6. BufferedReader atop BytesIO — read1 / peek / readline.
# Buffer sized larger than the payload so peek sees the next-line prefix
# in one fill.
_payload6 = b"line1\nline2\nline3\n"
_under = io.BytesIO(_payload6)
_br = io.BufferedReader(_under, buffer_size=64)
# readline returns the first line including trailing newline.
assert _br.readline() == b"line1\n", "BufferedReader.readline returns first line"
_ledger.append(1)
# peek looks ahead without consuming. CPython may return more than
# requested; the returned bytes start with the next available byte.
_peek = _br.peek(5)
assert isinstance(_peek, bytes), "peek returns bytes"
_ledger.append(1)
assert _peek.startswith(b"line2"), "peek prefix matches next line"
_ledger.append(1)
# peek does NOT consume — position is unchanged.
assert _br.read(6) == b"line2\n", "read after peek consumes 6 bytes (peek did not advance)"
_ledger.append(1)
# read1 reads at most n bytes from the underlying stream's buffer.
_rest = _br.read1(64)
assert isinstance(_rest, bytes), "read1 returns bytes"
_ledger.append(1)
assert _rest.startswith(b"line3"), "read1 returns the next available chunk"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: io {len(_ledger)} asserts")

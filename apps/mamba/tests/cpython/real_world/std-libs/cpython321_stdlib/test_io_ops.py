# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_io_ops"
# subject = "cpython321.test_io_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_io_ops.py"
# status = "filled"
# ///
"""cpython321.test_io_ops: execute CPython 3.12 seed test_io_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `io.StringIO` and `io.BytesIO`.
# Surface: write/getvalue, seek/tell, read after seek(0).
# Companion to stub/test_io.py — vendored unittest seed.
from io import StringIO, BytesIO
_ledger: list[int] = []

sio = StringIO()
sio.write("hello")
sio.write(" world")
assert sio.getvalue() == "hello world"; _ledger.append(1)

sio2 = StringIO("preloaded")
assert sio2.read() == "preloaded"; _ledger.append(1)

bio = BytesIO()
bio.write(b"abc")
bio.write(b"def")
assert bio.getvalue() == b"abcdef"; _ledger.append(1)

bio2 = BytesIO(b"\x00\x01\x02")
data = bio2.read()
assert data == b"\x00\x01\x02"; _ledger.append(1)

sio3 = StringIO("0123456789")
sio3.seek(5)
assert sio3.read() == "56789"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_io_ops {sum(_ledger)} asserts")

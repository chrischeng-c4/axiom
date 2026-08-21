# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_base64"
# subject = "cpython321.test_base64"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_base64.py"
# status = "filled"
# ///
"""cpython321.test_base64: execute CPython 3.12 seed test_base64"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
import base64

_ledger: list[int] = []

# b64encode / b64decode roundtrip
assert base64.b64encode(b"abc") == b"YWJj", "b64encode 'abc'"
_ledger.append(1)

assert base64.b64decode(b"YWJj") == b"abc", "b64decode 'YWJj'"
_ledger.append(1)

assert base64.b64encode(b"") == b"", "b64encode empty"
_ledger.append(1)

assert base64.b64decode(b"") == b"", "b64decode empty"
_ledger.append(1)

assert base64.b64encode(b"Aladdin:open sesame") == b"QWxhZGRpbjpvcGVuIHNlc2FtZQ==", "b64encode padded"
_ledger.append(1)

# urlsafe variant (uses '-' / '_' instead of '+' / '/')
assert base64.urlsafe_b64encode(b"\xfb\xff") == b"-_8=", "urlsafe encode preserves -_ alphabet"
_ledger.append(1)

assert base64.urlsafe_b64decode(b"-_8=") == b"\xfb\xff", "urlsafe decode roundtrip"
_ledger.append(1)

# b16
assert base64.b16encode(b"abc") == b"616263", "b16encode 'abc'"
_ledger.append(1)

assert base64.b16decode(b"616263") == b"abc", "b16decode '616263'"
_ledger.append(1)

# b32
assert base64.b32encode(b"abc") == b"MFRGG===", "b32encode 'abc'"
_ledger.append(1)

assert base64.b32decode(b"MFRGG===") == b"abc", "b32decode 'MFRGG==='"
_ledger.append(1)

# encodebytes / decodebytes (legacy, with trailing newline)
assert base64.encodebytes(b"abc") == b"YWJj\n", "encodebytes appends newline"
_ledger.append(1)

assert base64.decodebytes(b"YWJj\n") == b"abc", "decodebytes accepts newline"
_ledger.append(1)

# Multi-byte roundtrip
data = b"www.python.org"
assert base64.b64decode(base64.b64encode(data)) == data, "b64 roundtrip preserves bytes"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_base64 {sum(_ledger)} asserts")

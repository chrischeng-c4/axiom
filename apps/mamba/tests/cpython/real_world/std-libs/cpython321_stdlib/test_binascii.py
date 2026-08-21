# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_binascii"
# subject = "cpython321.test_binascii"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_binascii.py"
# status = "filled"
# ///
"""cpython321.test_binascii: execute CPython 3.12 seed test_binascii"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
import binascii

_ledger: list[int] = []

# hexlify converts bytes to lowercase ASCII hex
assert binascii.hexlify(b"hi") == b"6869", "hexlify('hi') == b'6869'"
_ledger.append(1)

# hexlify of empty input is empty
assert binascii.hexlify(b"") == b"", "hexlify(b'') == b''"
_ledger.append(1)

# hexlify covers every byte value in 0x00-0x0f
assert binascii.hexlify(bytes(range(16))) == b"000102030405060708090a0b0c0d0e0f", (
    "hexlify(range(16)) emits canonical lowercase hex"
)
_ledger.append(1)

# unhexlify is the inverse of hexlify
assert binascii.unhexlify(b"6869") == b"hi", "unhexlify('6869') == b'hi'"
_ledger.append(1)

# unhexlify roundtrip preserves arbitrary byte sequences
src = bytes(range(16))
assert binascii.unhexlify(binascii.hexlify(src)) == src, "unhexlify(hexlify(x)) == x"
_ledger.append(1)

# b2a_hex is the documented alias for hexlify
assert binascii.b2a_hex(b"AB") == b"4142", "b2a_hex('AB') == b'4142'"
_ledger.append(1)

# a2b_hex is the documented alias for unhexlify
assert binascii.a2b_hex(b"4142") == b"AB", "a2b_hex('4142') == b'AB'"
_ledger.append(1)

# b2a_base64 emits the standard base64 alphabet with trailing newline
assert binascii.b2a_base64(b"abc") == b"YWJj\n", "b2a_base64('abc') == b'YWJj\\n'"
_ledger.append(1)

# b2a_base64 of empty input is just a newline
assert binascii.b2a_base64(b"") == b"\n", "b2a_base64(b'') == b'\\n'"
_ledger.append(1)

# a2b_base64 inverts b2a_base64 (ignoring the newline)
assert binascii.a2b_base64(b"YWJj") == b"abc", "a2b_base64('YWJj') == b'abc'"
_ledger.append(1)

# base64 roundtrip preserves arbitrary byte sequences
payload = b"Hello, world!"
assert binascii.a2b_base64(binascii.b2a_base64(payload)) == payload, (
    "a2b_base64(b2a_base64(x)) == x"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_binascii {sum(_ledger)} asserts")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops"
# subject = "cpython321.test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops.py"
# status = "filled"
# ///
"""cpython321.test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops: execute CPython 3.12 seed test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 179: base64 b16 casefold + encodebytes multi-byte / multi-line +
# urlsafe full -/_ alphabet + b64 + / alphabet pinning
#
# Covers `base64` surfaces not asserted by test_base64_a85_roundtrip_ops
# (decode(encode(x))==x roundtrips) / test_base64_b85_ops / test_base64.py
# (which pins b16encode(b"abc")==b"616263" and one urlsafe \xfb\xff case
# only):
#   - b16encode / b16decode on longer multi-byte payloads (exact-value
#     pins, not just roundtrip)
#   - b16decode(casefold=True) — accepts lowercase hex
#   - encodebytes on longer payloads (legacy line-wrapping output that
#     terminates with newline) and trivial cases (empty / single byte)
#   - decodebytes on multi-line input (interior `\n` separators)
#   - urlsafe_b64encode / urlsafe_b64decode pinning the full `-` and `_`
#     alphabet on \xff\xff\xff (yielding all-`_`) and arbitrary binary
#     roundtrip
#   - b64encode / b64decode pinning the standard `+` / `/` alphabet on
#     \xfb\xff
import base64

_ledger = []

# --- b16encode / b16decode multi-byte exact values ---
assert base64.b16encode(b"hello world") == b"68656C6C6F20776F726C64"; _ledger.append(1)
assert base64.b16decode(b"68656C6C6F20776F726C64") == b"hello world"; _ledger.append(1)
assert base64.b16encode(b"\x00\x01\x02\xff") == b"000102FF"; _ledger.append(1)
assert base64.b16decode(b"000102FF") == b"\x00\x01\x02\xff"; _ledger.append(1)
assert base64.b16encode(b"\xde\xad\xbe\xef") == b"DEADBEEF"; _ledger.append(1)
assert base64.b16decode(b"DEADBEEF") == b"\xde\xad\xbe\xef"; _ledger.append(1)

# --- b16decode(casefold=True) accepts lowercase ---
assert base64.b16decode(b"68656c6c6f", casefold=True) == b"hello"; _ledger.append(1)
assert base64.b16decode(b"4142", casefold=True) == b"AB"; _ledger.append(1)
assert base64.b16decode(b"deadbeef", casefold=True) == b"\xde\xad\xbe\xef"; _ledger.append(1)

# --- b16 roundtrip on arbitrary binary ---
payload = b"\x00\x01\x02\x03\xfc\xfd\xfe\xff"
assert base64.b16decode(base64.b16encode(payload)) == payload; _ledger.append(1)

# --- encodebytes ---
# trivial cases
assert base64.encodebytes(b"") == b""; _ledger.append(1)
assert base64.encodebytes(b"x") == b"eA==\n"; _ledger.append(1)
assert base64.encodebytes(b"hello") == b"aGVsbG8=\n"; _ledger.append(1)
# longer payload — terminates with newline
out60 = base64.encodebytes(b"a" * 60)
assert out60.endswith(b"\n"); _ledger.append(1)
assert base64.decodebytes(out60) == b"a" * 60; _ledger.append(1)

# --- decodebytes accepts multi-line input ---
multi = b"YWFh\nYmJi\n"
assert base64.decodebytes(multi) == b"aaabbb"; _ledger.append(1)
# trailing newline only
assert base64.decodebytes(b"aGVsbG8=\n") == b"hello"; _ledger.append(1)
# decodebytes empty
assert base64.decodebytes(b"") == b""; _ledger.append(1)

# --- urlsafe_b64 — full -/_ alphabet ---
assert base64.urlsafe_b64encode(b"\xff\xff\xff") == b"____"; _ledger.append(1)
assert base64.urlsafe_b64decode(b"____") == b"\xff\xff\xff"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"") == b""; _ledger.append(1)
assert base64.urlsafe_b64encode(b"\xfb\xff") == b"-_8="; _ledger.append(1)
assert base64.urlsafe_b64decode(b"-_8=") == b"\xfb\xff"; _ledger.append(1)
# urlsafe roundtrip
binp = b"\x00\x01\x02\xfd\xfe\xff"
assert base64.urlsafe_b64decode(base64.urlsafe_b64encode(binp)) == binp; _ledger.append(1)

# --- b64encode / b64decode — standard +/ alphabet pinning ---
assert base64.b64encode(b"\xfb\xff") == b"+/8="; _ledger.append(1)
assert base64.b64decode(b"+/8=") == b"\xfb\xff"; _ledger.append(1)
# b64encode known fixed values
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64encode(b"") == b""; _ledger.append(1)
assert base64.b64decode(b"") == b""; _ledger.append(1)
# standard b64 of all-high bytes — covers +// dialect distinct from urlsafe
assert base64.b64encode(b"\xff\xff\xff") == b"////"; _ledger.append(1)
assert base64.b64decode(b"////") == b"\xff\xff\xff"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_base64_b16_casefold_encodebytes_urlsafe_alphabet_ops {sum(_ledger)} asserts")

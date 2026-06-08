# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_codecs_encode_decode_ops"
# subject = "cpython321.test_codecs_encode_decode_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_codecs_encode_decode_ops.py"
# status = "filled"
# ///
"""cpython321.test_codecs_encode_decode_ops: execute CPython 3.12 seed test_codecs_encode_decode_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for codecs.encode / codecs.decode
# and the str.encode / bytes.decode methods across ASCII, UTF-8,
# and Latin-1 codecs.
# Surface: codecs.encode(str, name) yields the bytes encoding;
# codecs.decode(bytes, name) yields the str decoding; the module-level
# functions agree with the bound str.encode / bytes.decode methods;
# ASCII round-trips a 7-bit string; UTF-8 round-trips a string with
# non-ASCII chars through the canonical multi-byte representation;
# Latin-1 round-trips any byte value 0-255 to its codepoint.
import codecs
_ledger: list[int] = []

# codecs.encode / decode against ASCII
assert codecs.encode("hello", "ascii") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello", "ascii") == "hello"; _ledger.append(1)

# codecs.encode / decode against UTF-8 for plain ASCII
assert codecs.encode("hello", "utf-8") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello", "utf-8") == "hello"; _ledger.append(1)

# UTF-8 of "café": the é is the two-byte sequence \xc3\xa9
assert codecs.encode("café", "utf-8") == b"caf\xc3\xa9"; _ledger.append(1)
assert codecs.decode(b"caf\xc3\xa9", "utf-8") == "café"; _ledger.append(1)

# str.encode method agrees with codecs.encode
assert "hello".encode("utf-8") == b"hello"; _ledger.append(1)
assert "café".encode("utf-8") == b"caf\xc3\xa9"; _ledger.append(1)

# bytes.decode method agrees with codecs.decode
assert b"hello".decode("utf-8") == "hello"; _ledger.append(1)
assert b"caf\xc3\xa9".decode("utf-8") == "café"; _ledger.append(1)

# Latin-1 round-trip of pure ASCII is identical to ASCII
assert "hello".encode("latin-1") == b"hello"; _ledger.append(1)
assert b"hello".decode("latin-1") == "hello"; _ledger.append(1)

# UTF-8 round-trip of an empty string yields empty bytes and back
assert "".encode("utf-8") == b""; _ledger.append(1)
assert b"".decode("utf-8") == ""; _ledger.append(1)

# Length comparison: ASCII chars are 1 byte each in UTF-8
ascii_bytes = "abc".encode("utf-8")
assert len(ascii_bytes) == 3; _ledger.append(1)

# Non-ASCII chars take multiple bytes in UTF-8
caffe_bytes = "café".encode("utf-8")
# c(1) + a(1) + f(1) + é(2) = 5 bytes
assert len(caffe_bytes) == 5; _ledger.append(1)

# Round-trip: encode then decode returns the original string
assert "hello world".encode("utf-8").decode("utf-8") == "hello world"; _ledger.append(1)
assert "καλημέρα".encode("utf-8").decode("utf-8") == "καλημέρα"; _ledger.append(1)

# Decoding bytes via codecs round-trips through the bound method
b1 = b"caf\xc3\xa9"
assert codecs.decode(b1, "utf-8") == b1.decode("utf-8"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_codecs_encode_decode_ops {sum(_ledger)} asserts")

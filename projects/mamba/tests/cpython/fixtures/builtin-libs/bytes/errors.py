# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes methods: documented exception paths (CPython 3.12 oracle)."""


# decode with bad encoding raises LookupError.
try:
    b"hi".decode("no_such_codec")
    print("bad_codec: no_raise")
except LookupError as e:
    print("bad_codec:", type(e).__name__, str(e)[:60])


# decode strict on invalid UTF-8 raises UnicodeDecodeError.
try:
    b"\xff\xfe".decode("utf-8")
    print("bad_utf8: no_raise")
except UnicodeDecodeError as e:
    print("bad_utf8:", type(e).__name__, str(e)[:60])


# bytes from non-bytelike with no encoding raises TypeError.
try:
    bytes("string with no encoding")  # type: ignore[call-overload]
    print("str_no_enc: no_raise")
except TypeError as e:
    print("str_no_enc:", type(e).__name__, str(e)[:60])


# bytes from str with bad encoding name raises LookupError.
try:
    bytes("hi", "no_such_codec")
    print("bad_codec_cons: no_raise")
except LookupError as e:
    print("bad_codec_cons:", type(e).__name__, str(e)[:60])


# index of missing substring raises ValueError.
try:
    b"abc".index(b"xyz")
    print("missing_idx: no_raise")
except ValueError as e:
    print("missing_idx:", type(e).__name__, str(e)[:60])


# Indexing OOR raises IndexError.
try:
    b"abc"[10]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# bytes + str raises TypeError.
try:
    b"a" + "b"  # type: ignore[operator]
    print("bytes_plus_str: no_raise")
except TypeError as e:
    print("bytes_plus_str:", type(e).__name__, str(e)[:60])


# bytes.fromhex with bad hex raises ValueError.
try:
    bytes.fromhex("xyz")
    print("bad_hex: no_raise")
except ValueError as e:
    print("bad_hex:", type(e).__name__, str(e)[:60])


# Multiplying bytes by non-int raises TypeError.
try:
    b"x" * "y"  # type: ignore[operator]
    print("times_str: no_raise")
except TypeError as e:
    print("times_str:", type(e).__name__, str(e)[:60])


# Membership of an int outside range(256) raises ValueError.
try:
    300 in b"abc"
    print("contains_oor: no_raise")
except ValueError as e:
    print("contains_oor:", type(e).__name__, str(e)[:60])


# Membership of a str in bytes raises TypeError.
try:
    "a" in b"abc"  # type: ignore[operator]
    print("contains_str: no_raise")
except TypeError as e:
    print("contains_str:", type(e).__name__, str(e)[:60])


# bytes += str raises TypeError (no implicit text concat).
try:
    bb = bytearray(b"abc")
    bb += "x"  # type: ignore[operator]
    print("iadd_str: no_raise")
except TypeError as e:
    print("iadd_str:", type(e).__name__, str(e)[:60])


# Assigning an out-of-range int into a bytearray slot raises ValueError.
try:
    bb = bytearray(b"abc")
    bb[0] = 256
    print("setitem_oor: no_raise")
except ValueError as e:
    print("setitem_oor:", type(e).__name__, str(e)[:60])


# pop() on an empty bytearray raises IndexError.
try:
    bytearray().pop()
    print("pop_empty: no_raise")
except IndexError as e:
    print("pop_empty:", type(e).__name__, str(e)[:60])


# translate with a table of wrong length raises ValueError.
try:
    bytearray(b"hi").translate(bytes(range(255)))
    print("bad_table: no_raise")
except ValueError as e:
    print("bad_table:", type(e).__name__, str(e)[:60])
